# GUI Development Tasks

## Current Issues to Fix

### High Priority
- [x] **Fix missing toolbar buttons** - Added visible buttons with borders and backgrounds
- [ ] **Add file drag-and-drop support** - Should be able to drag files into the left panel
- [x] **Fix file dialog integration** - Added debug output and proper error handling
- [x] **Implement proper file icons** - Different file types now have distinct colors and icons
- [x] **Update buttons to match GUI spec** - All 6 buttons (B1-B6) implemented per specification
- [x] **Add button state management** - Buttons enable/disable based on app state per spec

### Visual Design Issues
- [ ] **Match GUI1.png colors exactly** - Current colors don't match the mockup
- [x] **Fix button spacing and sizing** - Toolbar buttons properly spaced with correct sizes
- [ ] **Implement hover effects** - Buttons should have hover states
- [ ] **Fix file table alternating row colors** - Should match the mockup
- [ ] **Add proper window title bar** - Should show "RolyPoly" clearly
- [ ] **Fix donut chart visualization** - Should look more like the mockup

### Functional Issues
- [ ] **Connect real file paths** - Currently using display names instead of actual paths
- [ ] **Add progress indicators** - For compress/extract operations
- [ ] **Implement file selection** - Multiple file selection with checkboxes
- [x] **Add context menus framework** - Right-click on files for options
- [x] **Implement state transitions** - Proper handling of Empty/Building/Ready/Extracting states

### Missing Features
- [ ] **Copy path functionality** - Implement clipboard integration
- [ ] **Share functionality** - OS-level sharing integration
- [ ] **Settings dialog** - Configuration options
- [ ] **Tools dialog** - Archive verification, repair, etc.
- [ ] **Keyboard shortcuts** - Ctrl+O for open, etc.
- [ ] **Status bar** - Show current operation status

## Reference Images
- GUI1.png: Shows extract mode with blue "Extract" button
- GUI2.png: Shows compress mode with blue "Compress" button

## Key Design Elements
1. **Left Panel**: Dark background (#1a1a1a) with file list
2. **Right Panel**: Slightly lighter background (#2a2a2a) with archive info
3. **Toolbar**: Icon buttons at top of left panel
4. **File Table**: Headers with Name, Size, Type, Modified columns
5. **Archive Info**: Donut chart showing 314.1KB with 62% compression
6. **Primary Button**: Blue (#4a90e2) Extract/Compress button

## Current Status
- [x] Basic Slint UI framework setup
- [x] Window layout structure
- [ ] Visual design matching mockups
- [ ] Functional buttons and dialogs
- [ ] File management integration
