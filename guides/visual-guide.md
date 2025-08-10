# 🎨 Visual Guide - ePub Reader Library

## 📱 Main Interface

### Initial Screen (Empty Library)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader Library    [Translate to: Select Language ▼]    [+ Add Book] │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                    📚                                       │
│                                                                             │
│                            Your Library is Empty                           │
│                                                                             │
│                        Add your first ebook to get started                 │
│                                                                             │
│                            [Add Your First Book]                           │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Library with Books
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader Library    [Translate to: English ▼]          [+ Add Book]   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │     📖      │  │     📖      │  │     📖      │  │     📖      │        │
│  │             │  │             │  │             │  │             │        │
│  │   [COVER]   │  │   [COVER]   │  │   [COVER]   │  │   [COVER]   │        │
│  │             │  │             │  │             │  │             │        │
│  │             │  │             │  │             │  │             │        │
│  │🟢 COMPLETED │  │🟡 IN PROGRESS│  │🔘 NOT STARTED│  │🔴 FAILED    │        │
│  │             │  │             │  │             │  │             │        │
│  │ The Hobbit  │  │ 1984        │  │ Dune        │  │ Neuromancer │        │
│  │ J.R.R.      │  │ George      │  │ Frank       │  │ William     │        │
│  │ Tolkien     │  │ Orwell      │  │ Herbert     │  │ Gibson      │        │
│  │ English     │  │ English     │  │ English     │  │ English     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │     📖      │  │     📖      │  │     📖      │  │     📖      │        │
│  │   [COVER]   │  │   [COVER]   │  │   [COVER]   │  │   [COVER]   │        │
│  │🟢 COMPLETED │  │🔘 NOT STARTED│  │🟡 IN PROGRESS│  │🟢 COMPLETED │        │
│  │ Foundation  │  │ Hyperion    │  │ One Hundred │  │ Harry Potter│        │
│  │ Isaac       │  │ Dan         │  │ Years of    │  │ J.K.        │        │
│  │ Asimov      │  │ Simmons     │  │ Solitude    │  │ Rowling     │        │
│  │ English     │  │ English     │  │ Spanish     │  │ English     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Hover on Book (Available Actions)
```
┌─────────────┐
│     📖      │
│             │
│   [COVER]   │
│             │
│ ┌─────────┐ │ ← Overlay appears on hover
│ │ [Read]  │ │
│ │[Translate]│ │
│ └─────────┘ │
│🟢 COMPLETED │
│ The Hobbit  │
│ J.R.R.      │
│ Tolkien     │
│ English     │
└─────────────┘
```

## 🔄 Modals and Dialogs

### Modal: Ollama Setup Required
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                    ┌─────────────────────────────────┐                      │
│                    │                                 │                      │
│                    │        Ollama Setup Required    │                      │
│                    │                                 │                      │
│                    │  Ollama is not running. Please │                      │
│                    │  start Ollama to enable        │                      │
│                    │  translation features.         │                      │
│                    │                                 │                      │
│                    │  [Check Again] [Continue Without│                      │
│                    │                 Translation]    │                      │
│                    │                                 │                      │
│                    └─────────────────────────────────┘                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Modal: Translation in Progress
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                    ┌─────────────────────────────────┐                      │
│                    │                                 │                      │
│                    │        Translating Book         │                      │
│                    │                                 │                      │
│                    │ ████████████░░░░░░░░░░░░░░░░░░░ │                      │
│                    │              65%                │                      │
│                    │                                 │                      │
│                    │   Translating chapter 12 of 18 │                      │
│                    │                                 │                      │
│                    │           [Cancel]              │                      │
│                    │                                 │                      │
│                    └─────────────────────────────────┘                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Loading: Loading Library
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader Library    [Translate to: Select Language ▼]    [+ Add Book] │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                    ⟳                                       │
│                                                                             │
│                            Loading your library...                         │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 📖 Reader Interface

### Book Home Page (Cover)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                                                                             │
│                              ┌─────────────┐                               │
│                              │             │                               │
│                              │    BOOK     │                               │
│                              │   COVER     │                               │
│                              │             │                               │
│                              └─────────────┘                               │
│                                                                             │
│                              The Hobbit                                     │
│                             J.R.R. Tolkien                                  │
│                                                                             │
│                           Table of Contents                                 │
│                                                                             │
│                          • Chapter I - An Unexpected Party                 │
│                          • Chapter II - Roast Mutton                       │
│                          • Chapter III - A Short Rest                      │
│                          • Chapter IV - Over Hill and Under Hill           │
│                          • ...                                             │
│                                                                             │
│                            [Start Reading]                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Chapter Page
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                    Chapter I - An Unexpected Party                          │
│                                                                             │
│  [← Previous]              [Contents]              [Next →]                │
│                                                                             │
│ ┌─────────────────┐                                                         │
│ │ [A-] [A+] [🌓] [⛶] │ ← Reading controls                                     │
│ └─────────────────┘                                                         │
│                                                                             │
│     In a hole in the ground there lived a hobbit. Not a nasty, dirty,      │
│ wet hole, filled with the ends of worms and an oozy smell, nor yet a       │
│ dry, bare, sandy hole with nothing in it to sit down on or to eat: it      │
│ was a hobbit-hole, and that means comfort.                                 │
│                                                                             │
│     It had a perfectly round door like a porthole, painted green, with     │
│ a shiny yellow brass knob in the exact middle. The door opened on to a     │
│ tube-shaped hall like a tunnel: a very comfortable tunnel without smoke,   │
│ with panelled walls, and floors tiled and carpeted, provided with          │
│ polished chairs, and lots and lots of pegs for hats and coats - the        │
│ hobbit was fond of visitors.                                               │
│                                                                             │
│                              [Image, if any]                               │
│                                                                             │
│     Chapter text continuation...                                           │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ ████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 75%    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Reading Controls (Expanded)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│ ┌─────────────────┐                                                         │
│ │ [A-] [A+] [🌓] [⛶] │ ← Click each button                                   │
│ └─────────────────┘                                                         │
│   │    │    │    │                                                          │
│   │    │    │    └── Fullscreen                                             │
│   │    │    └─────── Toggle theme (light/dark)                              │
│   │    └──────────── Increase font                                          │
│   └───────────────── Decrease font                                          │
│                                                                             │
│ Control effects:                                                            │
│                                                                             │
│ Small font:  This is sample text                                           │
│ Normal font: This is sample text                                           │
│ Large font:  This is sample text                                           │
│                                                                             │
│ Light theme: ┌─────────────────┐                                           │
│              │ Black text      │                                           │
│              │ White background│                                           │
│              └─────────────────┘                                           │
│                                                                             │
│ Dark theme:  ┌─────────────────┐                                           │
│              │ White text      │                                           │
│              │ Black background│                                           │
│              └─────────────────┘                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 🎯 Visual Usage Flow

### 1. Add Book
```
[Click "Add Book"] → [File Selector] → [Processing] → [Book in Library]

┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ + Add Book  │ →  │ 📁 Select   │ →  │ ⟳ Loading   │ →  │ 📖 Book     │
│             │    │ file.epub   │    │ Processing  │    │ Added       │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### 2. Translate Book
```
[Select Language] → [Hover on Book] → [Click "Translate"] → [Wait] → [Complete]

┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Translate   │ →  │ 📖 Hover    │ →  │ Translate   │ →  │ 🟡 65%      │ →  │ 🟢 Complete │
│ to: English │    │ [Translate] │    │ Progress    │    │ Translating │    │ Translated  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### 3. Read Book
```
[Hover on Book] → [Click "Read"] → [Browser Opens] → [Reading]

┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ 📖 Hover    │ →  │ 🌐 Browser  │ →  │ 📄 Cover    │ →  │ 📖 Chapter  │
│ [Read]      │    │ Opens       │    │ & Contents  │    │ Content     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

## 🎨 Book Visual States

### Translation Status
```
🔘 NOT STARTED     🟡 IN PROGRESS     🟢 COMPLETED     🔴 FAILED
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ 📖 [COVER]  │    │ 📖 [COVER]  │    │ 📖 [COVER]  │    │ 📖 [COVER]  │
│🔘 NOT STARTED│    │🟡 IN PROGRESS│    │🟢 COMPLETED │    │🔴 FAILED    │
│ Title       │    │ Title       │    │ Title       │    │ Title       │
│ Author      │    │ Author      │    │ Author      │    │ Author      │
│ Language    │    │ Language    │    │ Language    │    │ Language    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### Available Actions by Status
```
NOT STARTED:           IN PROGRESS:           COMPLETED:             FAILED:
┌─────────────┐        ┌─────────────┐        ┌─────────────┐        ┌─────────────┐
│ [Read]      │        │ [Read]      │        │ [Read]      │        │ [Read]      │
│ [Translate] │        │ [Cancel]    │        │ [Re-translate]│       │ [Retry]     │
└─────────────┘        └─────────────┘        └─────────────┘        └─────────────┘
```

## 📱 Responsiveness

### Desktop (Large Screen)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Header with Logo, Language Selector and Add Book Button                    │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐  ← 5 books per row                          │
│ │ 📖│ │ 📖│ │ 📖│ │ 📖│ │ 📖│                                             │
│ └───┘ └───┘ └───┘ └───┘ └───┘                                             │
│ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐                                             │
│ │ 📖│ │ 📖│ │ 📖│ │ 📖│ │ 📖│                                             │
│ └───┘ └───┘ └───┘ └───┘ └───┘                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Tablet (Medium Screen)
```
┌─────────────────────────────────────────────────────┐
│ Stacked header                                      │
│ Logo                                                │
│ Language Selector                                   │
│ Add Book Button                                     │
├─────────────────────────────────────────────────────┤
│ ┌─────┐ ┌─────┐ ┌─────┐  ← 3 books per row         │
│ │ 📖  │ │ 📖  │ │ 📖  │                            │
│ └─────┘ └─────┘ └─────┘                            │
│ ┌─────┐ ┌─────┐ ┌─────┐                            │
│ │ 📖  │ │ 📖  │ │ 📖  │                            │
│ └─────┘ └─────┘ └─────┘                            │
└─────────────────────────────────────────────────────┘
```

### Mobile (Small Screen)
```
┌─────────────────────────────┐
│ Vertical Header             │
│ 📚 ePub Reader             │
│ [Translate to: EN ▼]       │
│ [+ Add Book]               │
├─────────────────────────────┤
│ ┌─────────┐ ┌─────────┐    │ ← 2 books per row
│ │   📖    │ │   📖    │    │
│ │ [COVER] │ │ [COVER] │    │
│ │ Title   │ │ Title   │    │
│ └─────────┘ └─────────┘    │
│ ┌─────────┐ ┌─────────┐    │
│ │   📖    │ │   📖    │    │
│ │ [COVER] │ │ [COVER] │    │
│ │ Title   │ │ Title   │    │
│ └─────────┘ └─────────┘    │
└─────────────────────────────┘
```

## 🎯 Visual Indicators

### Progress Bar (Translation)
```
0%   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
25%  ████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
50%  ████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
75%  ████████████████████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░
100% ████████████████████████████████████████████████████████████████████████████
```

### Progress Bar (Reading)
```
Chapter start:   ████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Chapter middle:  ████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░
Chapter end:     ████████████████████████████████████████████████████████████
```

### Loading Spinner
```
Frame 1: ⠋ Loading...
Frame 2: ⠙ Loading...
Frame 3: ⠹ Loading...
Frame 4: ⠸ Loading...
Frame 5: ⠼ Loading...
Frame 6: ⠴ Loading...
Frame 7: ⠦ Loading...
Frame 8: ⠧ Loading...
```

---

## 🎨 Color Palette

### Dark Theme (Default)
```
┌─────────────────────────────────────┐
│ Background: #1a1a1a (Soft black)   │
│ Cards: #2d2d2d (Dark gray)         │
│ Text: #ffffff (White)               │
│ Secondary: #b0b0b0 (Light gray)     │
│ Primary: #3498db (Blue)             │
│ Success: #27ae60 (Green)            │
│ Warning: #f39c12 (Yellow)           │
│ Error: #e74c3c (Red)                │
│ Info: #95a5a6 (Gray)                │
└─────────────────────────────────────┘
```

### Light Theme (Reader)
```
┌─────────────────────────────────────┐
│ Background: #f8f9fa (Soft white)   │
│ Cards: #ffffff (White)              │
│ Text: #333333 (Soft black)          │
│ Secondary: #7f8c8d (Medium gray)    │
│ Primary: #2980b9 (Dark blue)        │
│ Borders: #ecf0f1 (Very light gray) │
└─────────────────────────────────────┘
```

---

**🎨 This visual guide helps understand the interface and application flows intuitively!**