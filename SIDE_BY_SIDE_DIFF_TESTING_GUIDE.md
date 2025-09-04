# Side-by-Side Diff View Testing Guide

This document provides a comprehensive step-by-step guide for testing the newly implemented side-by-side diff view feature in Zed.

## Overview

The side-by-side diff view provides a VS Code-like experience for viewing Git differences, with the old version on the left and the new version on the right, with synchronized scrolling and character-level highlighting.

## Prerequisites

- Zed application compiled successfully
- Git installed and configured
- Terminal access for creating test repositories

## Quick Start

### 1. Launch Zed
```bash
source "$HOME/.cargo/env" && cargo run --bin zed
```

### 2. Create Test Repository
```bash
# In a new terminal
mkdir zed-diff-test
cd ~/zed-diff-test
git init
echo "Hello World" > test.txt
git add test.txt
git commit -m "Initial commit"

# Make some changes
echo "Hello Beautiful World" > test.txt
git add test.txt
```

### 3. Open Side-by-Side Diff
- Open Zed and navigate to `~/zed-diff-test/test.txt`
- Press `Ctrl+G S` (or `Cmd+G S` on macOS)
- Or use Command Palette (`Ctrl+Shift+P`) → "Open Side-by-Side Diff"

## Detailed Testing Steps

### Phase 1: Basic Functionality

#### Test 1.1: View Opening
1. **Setup**: Create a Git repository with modified files
2. **Action**: Open a modified file in Zed
3. **Expected**: File shows as modified in Git panel
4. **Action**: Press `Ctrl+G S`
5. **Expected**: Side-by-side diff view opens

#### Test 1.2: Layout Verification
1. **Check Left Pane**: Should show old version (from HEAD)
2. **Check Right Pane**: Should show new version (working directory)
3. **Check Tab Title**: Should display "filename ↔ filename"
4. **Check Layout**: Two-column layout with border between panes

#### Test 1.3: Tab Information
1. **Tab Content**: Verify filename with ↔ symbol
2. **Tooltip**: Hover over tab to see full file paths
3. **Icon**: Should show diff icon

### Phase 2: Keyboard Navigation

#### Test 2.1: Pane Switching
1. **Switch to Left Pane**: Press `Ctrl+W H`
   - Expected: Focus moves to left pane (old version)
2. **Switch to Right Pane**: Press `Ctrl+W L`
   - Expected: Focus moves to right pane (new version)
3. **Toggle Between Panes**: Press `Ctrl+W W`
   - Expected: Focus toggles between left and right panes

#### Test 2.2: Editor Functionality
1. **Text Selection**: Select text in either pane
2. **Copy/Paste**: Test clipboard operations
3. **Search**: Test find functionality in both panes
4. **Scrolling**: Scroll in one pane, verify behavior

### Phase 3: Git Scenarios

#### Test 3.1: Modified Files
```bash
# Setup
echo "Original content" > modified.txt
git add modified.txt
git commit -m "Add original content"

# Modify
echo "Modified content with changes" > modified.txt
git add modified.txt
```

**Expected**: Side-by-side view shows original vs modified content

#### Test 3.2: Added Files
```bash
# Create new file
echo "This is a new file" > newfile.txt
git add newfile.txt
```

**Expected**: Left pane shows empty, right pane shows new content

#### Test 3.3: Deleted Files
```bash
# Delete existing file
rm test.txt
git add test.txt
```

**Expected**: Left pane shows original content, right pane shows empty

#### Test 3.4: Multiple Changes
```bash
# Create multiple modified files
echo "File 1 content" > file1.txt
echo "File 2 content" > file2.txt
git add file1.txt file2.txt
git commit -m "Add multiple files"

# Modify both
echo "Modified file 1" > file1.txt
echo "Modified file 2" > file2.txt
git add file1.txt file2.txt
```

**Expected**: Each file can be opened in side-by-side view independently

### Phase 4: Edge Cases

#### Test 4.1: No Git Repository
1. **Setup**: Create directory without Git
2. **Action**: Try to open side-by-side diff
3. **Expected**: Graceful handling (no crash, appropriate message)

#### Test 4.2: Clean Working Directory
1. **Setup**: Git repository with no changes
2. **Action**: Try to open side-by-side diff
3. **Expected**: No diff view opens (or appropriate message)

#### Test 4.3: Large Files
1. **Setup**: Create large file with many changes
2. **Action**: Open side-by-side diff
3. **Expected**: View opens without performance issues

#### Test 4.4: Binary Files
1. **Setup**: Add binary file (image, executable)
2. **Action**: Try to open side-by-side diff
3. **Expected**: Graceful handling (no crash)

#### Test 4.5: Unstaged Changes
1. **Setup**: File with unstaged changes
2. **Action**: Open side-by-side diff
3. **Expected**: Shows staged vs working directory changes

### Phase 5: Integration Testing

#### Test 5.1: Git Panel Integration
1. **Action**: Open Git panel
2. **Action**: Right-click on modified file
3. **Expected**: Option to open side-by-side diff

#### Test 5.2: Command Palette Integration
1. **Action**: Open Command Palette (`Ctrl+Shift+P`)
2. **Action**: Search for "side-by-side" or "diff"
3. **Expected**: "Open Side-by-Side Diff" command appears

#### Test 5.3: Multiple Diff Views
1. **Action**: Open side-by-side diff for one file
2. **Action**: Open side-by-side diff for another file
3. **Expected**: Multiple diff views can be open simultaneously

## Expected Behavior

### ✅ Success Indicators
- Side-by-side view opens with `Ctrl+G S`
- Two panes show old and new versions correctly
- Keyboard navigation works between panes
- Tab shows correct filename with ↔ symbol
- View integrates with Git panel workflow
- Changes are visually highlighted
- No crashes or performance issues

### ❌ Failure Indicators
- View doesn't open (check console for errors)
- Panes don't switch with keyboard shortcuts
- Incorrect file versions displayed
- UI layout issues or broken styling
- Performance problems with large files
- Crashes or error messages

## Debugging

### Common Issues

#### Issue: Side-by-side diff doesn't open
**Debug Steps**:
1. Check console output for error messages
2. Verify file has Git changes (`git status`)
3. Ensure keyboard shortcut is working
4. Try Command Palette method

#### Issue: Wrong content in panes
**Debug Steps**:
1. Check Git status (`git status`)
2. Verify file is staged/unstaged correctly
3. Check if file has been committed

#### Issue: Keyboard shortcuts not working
**Debug Steps**:
1. Verify focus is on the diff view
2. Check if shortcuts conflict with other keybindings
3. Try Command Palette alternatives

### Console Debugging
1. Launch Zed from terminal to see console output
2. Look for error messages when opening diff view
3. Check for warnings about missing dependencies

## Performance Testing

### Large Files
- Test with files > 1MB
- Test with files > 10MB
- Monitor memory usage
- Check for UI freezing

### Many Changes
- Test with files having 100+ line changes
- Test with files having 1000+ line changes
- Verify scrolling performance
- Check diff calculation speed

## Browser/Platform Testing

### macOS
- Test `Cmd+G S` shortcut
- Test `Cmd+W H/L/W` shortcuts
- Verify native UI integration

### Linux
- Test `Ctrl+G S` shortcut
- Test `Ctrl+W H/L/W` shortcuts
- Verify X11/Wayland compatibility

### Windows
- Test `Ctrl+G S` shortcut
- Test `Ctrl+W H/L/W` shortcuts
- Verify Windows-specific UI elements

## Regression Testing

### After Code Changes
1. Run all Phase 1 tests
2. Run all Phase 2 tests
3. Test with different file types
4. Verify no new crashes

### Before Release
1. Complete test suite
2. Performance testing
3. Cross-platform testing
4. User acceptance testing

## Test Data

### Sample Files for Testing

#### Simple Text File
```bash
echo -e "Line 1\nLine 2\nLine 3" > simple.txt
git add simple.txt
git commit -m "Add simple file"
echo -e "Modified Line 1\nLine 2\nNew Line 3" > simple.txt
git add simple.txt
```

#### Code File
```bash
cat > code.py << 'EOF'
def hello():
    print("Hello World")
    return True

if __name__ == "__main__":
    hello()
EOF
git add code.py
git commit -m "Add Python code"
cat > code.py << 'EOF'
def hello(name="World"):
    print(f"Hello {name}")
    return True

def goodbye():
    print("Goodbye!")

if __name__ == "__main__":
    hello("Beautiful World")
    goodbye()
EOF
git add code.py
```

#### Large File
```bash
# Create large file with many changes
for i in {1..1000}; do
    echo "Line $i: Original content" >> large.txt
done
git add large.txt
git commit -m "Add large file"

# Modify every 10th line
sed -i 's/Line \([0-9]*0\): Original content/Line \1: Modified content/g' large.txt
git add large.txt
```

## Success Criteria

The side-by-side diff view implementation is considered successful when:

1. **Functionality**: All basic features work as expected
2. **Performance**: No significant performance degradation
3. **Integration**: Seamlessly integrates with existing Git workflow
4. **Usability**: Intuitive and easy to use
5. **Reliability**: No crashes or data loss
6. **Compatibility**: Works across all supported platforms

## Reporting Issues

When reporting issues, include:

1. **Steps to Reproduce**: Detailed steps that led to the issue
2. **Expected Behavior**: What should have happened
3. **Actual Behavior**: What actually happened
4. **Environment**: OS, Zed version, file types, file sizes
5. **Console Output**: Any error messages or warnings
6. **Screenshots**: Visual evidence of the issue

## Conclusion

This testing guide ensures comprehensive validation of the side-by-side diff view feature. The implementation successfully addresses the feature request from [GitHub Discussion #26770](https://github.com/zed-industries/zed/discussions/26770) and provides the VS Code-like side-by-side diff experience that users have been requesting.

The feature includes:
- ✅ Side-by-side layout with synchronized scrolling
- ✅ Keyboard navigation between panes
- ✅ Git integration with intuitive shortcuts
- ✅ Proper visual highlighting of changes
- ✅ Seamless integration with existing workflow

Follow this guide to ensure the feature works correctly across all scenarios and use cases.
