//! SideBySideDiffView provides a side-by-side UI for displaying Git differences.

use anyhow::Result;
use buffer_diff::{BufferDiff, BufferDiffSnapshot};
use editor::{Editor, EditorEvent, MultiBuffer};
use futures::{FutureExt, select_biased};
use gpui::{
    AnyElement, AnyView, App, AppContext as _, AsyncApp, Context, Entity, EventEmitter,
    FocusHandle, Focusable, IntoElement, ParentElement, Render, Styled, Task, Window,
};
use language::Buffer;
use project::Project;
use std::{
    any::{Any, TypeId},
    pin::pin,
    sync::Arc,
    time::Duration,
};
use text::{Anchor, Point};
use ui::{Color, Icon, IconName, Label, LabelCommon as _, SharedString};
use util::paths::PathExt as _;
use workspace::{
    Item, ItemHandle as _, ItemNavHistory, ToolbarItemLocation, Workspace,
    item::{BreadcrumbText, ItemEvent, SaveOptions, TabContentParams},
    searchable::SearchableItemHandle,
};

pub struct SideBySideDiffView {
    left_editor: Entity<Editor>,
    right_editor: Entity<Editor>,
    old_buffer: Entity<Buffer>,
    new_buffer: Entity<Buffer>,
    diff: Entity<BufferDiff>,
    buffer_changes_tx: watch::Sender<()>,
    _recalculate_diff_task: Task<Result<()>>,
    focused_pane: FocusedPane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedPane {
    Left,
    Right,
}

const RECALCULATE_DIFF_DEBOUNCE: Duration = Duration::from_millis(250);

impl SideBySideDiffView {
    pub fn open(
        old_buffer: Entity<Buffer>,
        new_buffer: Entity<Buffer>,
        workspace: &Workspace,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Result<Entity<Self>>> {
        let workspace = workspace.weak_handle();
        window.spawn(cx, async move |cx| {
            let project = workspace.update(cx, |workspace, _| workspace.project().clone())?;
            let buffer_diff = build_buffer_diff(&old_buffer, &new_buffer, cx).await?;

            workspace.update_in(cx, |workspace, window, cx| {
                let diff_view = cx.new(|cx| {
                    SideBySideDiffView::new(
                        old_buffer,
                        new_buffer,
                        buffer_diff,
                        project.clone(),
                        window,
                        cx,
                    )
                });

                let pane = workspace.active_pane();
                pane.update(cx, |pane, cx| {
                    pane.add_item(Box::new(diff_view.clone()), true, true, None, window, cx);
                });

                diff_view
            })
        })
    }

    pub fn new(
        old_buffer: Entity<Buffer>,
        new_buffer: Entity<Buffer>,
        diff: Entity<BufferDiff>,
        project: Entity<Project>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Create left editor for old buffer
        let left_multibuffer = cx.new(|cx| {
            let mut multibuffer = MultiBuffer::singleton(old_buffer.clone(), cx);
            multibuffer.add_diff(diff.clone(), cx);
            multibuffer
        });
        let left_editor = cx.new(|cx| {
            let mut editor = Editor::for_multibuffer(
                left_multibuffer.clone(),
                Some(project.clone()),
                window,
                cx,
            );
            editor.start_temporary_diff_override();
            editor.disable_diagnostics(cx);
            editor.set_expand_all_diff_hunks(cx);
            editor.set_render_diff_hunk_controls(
                Arc::new(|_, _, _, _, _, _, _, _| gpui::Empty.into_any_element()),
                cx,
            );
            editor
        });

        // Create right editor for new buffer
        let right_multibuffer = cx.new(|cx| {
            let mut multibuffer = MultiBuffer::singleton(new_buffer.clone(), cx);
            multibuffer.add_diff(diff.clone(), cx);
            multibuffer
        });
        let right_editor = cx.new(|cx| {
            let mut editor = Editor::for_multibuffer(
                right_multibuffer.clone(),
                Some(project.clone()),
                window,
                cx,
            );
            editor.start_temporary_diff_override();
            editor.disable_diagnostics(cx);
            editor.set_expand_all_diff_hunks(cx);
            editor.set_render_diff_hunk_controls(
                Arc::new(|_, _, _, _, _, _, _, _| gpui::Empty.into_any_element()),
                cx,
            );
            editor
        });

        // TODO: Implement synchronized scrolling
        // For now, we'll skip scroll synchronization to get basic functionality working

        let (buffer_changes_tx, mut buffer_changes_rx) = watch::channel(());

        for buffer in [&old_buffer, &new_buffer] {
            cx.subscribe(buffer, move |this, _, event, _| match event {
                language::BufferEvent::Edited
                | language::BufferEvent::LanguageChanged
                | language::BufferEvent::Reparsed => {
                    this.buffer_changes_tx.send(()).ok();
                }
                _ => {}
            })
            .detach();
        }

        Self {
            left_editor,
            right_editor,
            old_buffer,
            new_buffer,
            diff: diff.clone(),
            buffer_changes_tx,
            focused_pane: FocusedPane::Left,
            _recalculate_diff_task: cx.spawn(async move |this, cx| {
                while buffer_changes_rx.recv().await.is_ok() {
                    loop {
                        let mut timer = cx
                            .background_executor()
                            .timer(RECALCULATE_DIFF_DEBOUNCE)
                            .fuse();
                        let mut recv = pin!(buffer_changes_rx.recv().fuse());
                        select_biased! {
                            _ = timer => break,
                            _ = recv => continue,
                        }
                    }

                    log::trace!("start recalculating side-by-side diff");
                    let (old_snapshot, new_snapshot) = this.update(cx, |this, cx| {
                        (
                            this.old_buffer.read(cx).snapshot(),
                            this.new_buffer.read(cx).snapshot(),
                        )
                    })?;
                    let diff_snapshot = cx
                        .update(|cx| {
                            BufferDiffSnapshot::new_with_base_buffer(
                                new_snapshot.text.clone(),
                                Some(old_snapshot.text().into()),
                                old_snapshot,
                                cx,
                            )
                        })?
                        .await;
                    diff.update(cx, |diff, cx| {
                        diff.set_snapshot(diff_snapshot, &new_snapshot, cx)
                    })?;
                    log::trace!("finish recalculating side-by-side diff");
                }
                Ok(())
            }),
        }
    }

    pub fn switch_to_left_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focused_pane = FocusedPane::Left;
        window.focus(&self.left_editor.focus_handle(cx));
    }

    pub fn switch_to_right_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focused_pane = FocusedPane::Right;
        window.focus(&self.right_editor.focus_handle(cx));
    }

    pub fn toggle_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.focused_pane {
            FocusedPane::Left => self.switch_to_right_pane(window, cx),
            FocusedPane::Right => self.switch_to_left_pane(window, cx),
        }
    }
}

async fn build_buffer_diff(
    old_buffer: &Entity<Buffer>,
    new_buffer: &Entity<Buffer>,
    cx: &mut AsyncApp,
) -> Result<Entity<BufferDiff>> {
    let old_buffer_snapshot = old_buffer.read_with(cx, |buffer, _| buffer.snapshot())?;
    let new_buffer_snapshot = new_buffer.read_with(cx, |buffer, _| buffer.snapshot())?;

    let diff_snapshot = cx
        .update(|cx| {
            BufferDiffSnapshot::new_with_base_buffer(
                new_buffer_snapshot.text.clone(),
                Some(old_buffer_snapshot.text().into()),
                old_buffer_snapshot,
                cx,
            )
        })?
        .await;

    cx.new(|cx| {
        let mut diff = BufferDiff::new(&new_buffer_snapshot.text, cx);
        diff.set_snapshot(diff_snapshot, &new_buffer_snapshot.text, cx);
        diff
    })
}

impl EventEmitter<EditorEvent> for SideBySideDiffView {}

impl Focusable for SideBySideDiffView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match self.focused_pane {
            FocusedPane::Left => self.left_editor.focus_handle(cx),
            FocusedPane::Right => self.right_editor.focus_handle(cx),
        }
    }
}

impl Item for SideBySideDiffView {
    type Event = EditorEvent;

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Diff).color(Color::Muted))
    }

    fn tab_content(&self, params: TabContentParams, _window: &Window, cx: &App) -> AnyElement {
        Label::new(self.tab_content_text(params.detail.unwrap_or_default(), cx))
            .color(if params.selected {
                Color::Default
            } else {
                Color::Muted
            })
            .into_any_element()
    }

    fn tab_content_text(&self, _detail: usize, cx: &App) -> SharedString {
        let title_text = |buffer: &Entity<Buffer>| {
            buffer
                .read(cx)
                .file()
                .and_then(|file| {
                    Some(
                        file.full_path(cx)
                            .file_name()?
                            .to_string_lossy()
                            .to_string(),
                    )
                })
                .unwrap_or_else(|| "untitled".into())
        };
        let old_filename = title_text(&self.old_buffer);
        let new_filename = title_text(&self.new_buffer);

        format!("{old_filename} ↔ {new_filename}").into()
    }

    fn tab_tooltip_text(&self, cx: &App) -> Option<ui::SharedString> {
        let path = |buffer: &Entity<Buffer>| {
            buffer
                .read(cx)
                .file()
                .map(|file| file.full_path(cx).compact().to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".into())
        };
        let old_path = path(&self.old_buffer);
        let new_path = path(&self.new_buffer);

        Some(format!("{old_path} ↔ {new_path}").into())
    }

    fn to_item_events(event: &EditorEvent, f: impl FnMut(ItemEvent)) {
        Editor::to_item_events(event, f)
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("Side-by-Side Diff View Opened")
    }

    fn deactivated(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.left_editor
            .update(cx, |editor, cx| editor.deactivated(window, cx));
        self.right_editor
            .update(cx, |editor, cx| editor.deactivated(window, cx));
    }

    fn is_singleton(&self, _: &App) -> bool {
        false
    }

    fn act_as_type<'a>(
        &'a self,
        type_id: TypeId,
        self_handle: &'a Entity<Self>,
        _: &'a App,
    ) -> Option<AnyView> {
        if type_id == TypeId::of::<Self>() {
            Some(self_handle.to_any())
        } else if type_id == TypeId::of::<Editor>() {
            // Return the currently focused editor
            match self.focused_pane {
                FocusedPane::Left => Some(self.left_editor.to_any()),
                FocusedPane::Right => Some(self.right_editor.to_any()),
            }
        } else {
            None
        }
    }

    fn as_searchable(&self, _: &Entity<Self>) -> Option<Box<dyn SearchableItemHandle>> {
        // Return the currently focused editor for search
        match self.focused_pane {
            FocusedPane::Left => Some(Box::new(self.left_editor.clone())),
            FocusedPane::Right => Some(Box::new(self.right_editor.clone())),
        }
    }

    fn for_each_project_item(
        &self,
        cx: &App,
        f: &mut dyn FnMut(gpui::EntityId, &dyn project::ProjectItem),
    ) {
        self.left_editor.for_each_project_item(cx, f);
        self.right_editor.for_each_project_item(cx, f);
    }

    fn set_nav_history(
        &mut self,
        nav_history: ItemNavHistory,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Set navigation history on both editors
        // Note: We can't clone ItemNavHistory, so we'll set it on the right editor only
        self.right_editor.update(cx, |editor, _| {
            editor.set_nav_history(Some(nav_history));
        });
    }

    fn navigate(
        &mut self,
        data: Box<dyn Any>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        // Try navigation on both editors
        // Note: We can't clone Box<dyn Any>, so we'll try the right editor first
        self.right_editor
            .update(cx, |editor, cx| editor.navigate(data, window, cx))
    }

    fn breadcrumb_location(&self, _: &App) -> ToolbarItemLocation {
        ToolbarItemLocation::PrimaryLeft
    }

    fn breadcrumbs(&self, theme: &theme::Theme, cx: &App) -> Option<Vec<BreadcrumbText>> {
        // Return breadcrumbs from the currently focused editor
        match self.focused_pane {
            FocusedPane::Left => self.left_editor.breadcrumbs(theme, cx),
            FocusedPane::Right => self.right_editor.breadcrumbs(theme, cx),
        }
    }

    fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.left_editor.update(cx, |editor, cx| {
            editor.added_to_workspace(workspace, window, cx)
        });
        self.right_editor.update(cx, |editor, cx| {
            editor.added_to_workspace(workspace, window, cx)
        });
    }

    fn can_save(&self, cx: &App) -> bool {
        // The right editor handles the new buffer, so delegate to it
        self.right_editor.read(cx).can_save(cx)
    }

    fn save(
        &mut self,
        options: SaveOptions,
        project: Entity<Project>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Result<()>> {
        // Delegate saving to the right editor, which manages the new buffer
        self.right_editor
            .update(cx, |editor, cx| editor.save(options, project, window, cx))
    }
}

impl Render for SideBySideDiffView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // Create a horizontal split view with the two editors
        ui::div()
            .flex()
            .h_full()
            .child(
                ui::div()
                    .flex_1()
                    .border_r_1()
                    .child(self.left_editor.clone()),
            )
            .child(ui::div().flex_1().child(self.right_editor.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use project::{FakeFs, Project};
    use settings::{Settings, SettingsStore};
    use util::{path, TryFutureExt};
    use workspace::Workspace;

    fn init_test(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let settings_store = SettingsStore::test(cx);
            cx.set_global(settings_store);
            language::init(cx);
            Project::init_settings(cx);
            workspace::init_settings(cx);
            editor::init_settings(cx);
            theme::ThemeSettings::register(cx)
        });
    }

    // TODO: Add proper tests once the basic functionality is working
}
