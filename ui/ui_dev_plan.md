# RolyPoly Compressor UI Development Plan

## 1. Project Goal

To create a modern, intuitive user interface for the RolyPoly compression utility based on the provided screenshots. The UI will be built using web technologies (HTML, CSS, JavaScript) and integrated into the main application using Tauri or Flutter.

## 2. Key Views & Components

The UI will consist of three main views:

### View 1: Main Drag-and-Drop Window

This is the primary window the user interacts with.

-   **File:** `ui/index.html`
-   **Layout:** A simple, dark-themed window with two main sections.
-   **Components:**
    -   **Top Bar:**
        -   Contains a dropdown menu to select the compression format (e.g., ZIP, 7Z, BZIP2).
    -   **Compression Drop Zone:**
        -   Large, clearly marked area for users to drag and drop files/folders to be compressed.
        -   Will feature a prominent icon (e.g., a TGZ file icon).
        -   Label: "Drop here to compress".
    -   **Extraction Drop Zone:**
        -   A secondary area for dropping archives to be extracted.
        -   Label: "Drop here to extract".
-   **Styling:**
    -   Dark background (`#282828`).
    -   Minimalist aesthetic.
    -   Clear visual distinction between the two drop zones.

### View 2: Advanced Compression Options

This view appears when a user needs more control over a compression task. It could be a modal dialog or a separate view.

-   **File:** `ui/settings.html` (or integrated into `index.html` as a modal)
-   **Layout:** A form-based layout.
-   **Components:**
    -   **Compression Method Slider:** An `<input type="range">` allowing selection from "Store" to "Slow".
    -   **Split Dropdown:** A `<select>` menu for archive splitting options (e.g., 50 GB Blu-ray).
    -   **Password Fields:** Two `<input type="password">` fields for setting and confirming a password, with validation indicators.
    -   **Checkboxes for Options:**
        -   Encrypt filenames
        -   Solid archive
        -   Exclude Mac resource forks
        -   Verify compression integrity
        -   Delete file(s) after compression
        -   Archive items separately
-   **Styling:** Consistent dark theme, with clear labels and interactive elements.

### View 3: Application Preferences

A dedicated window for setting global application preferences.

-   **File:** `ui/settings.html`
-   **Layout:** A tabbed interface to organize different settings categories.
-   **Components:**
    -   **Tab Navigation:** Buttons for General, Appearance, Compression, Extraction, etc.
    -   **Compression Tab (as per screenshot):**
        -   **Default Format:** Dropdown.
        -   **Default Method:** Dropdown.
        -   **Save to Location:** Dropdown.
        -   **Name of new files:** Dropdown.
        -   Various checkboxes for fine-tuning default behaviors (e.g., "Verify compression integrity", "Exclude Mac resource forks").
-   **Styling:**
    -   Clear visual indication of the active tab.
    -   Grouped and labeled settings for readability.

## 3. Technology Stack

-   **Framework:** Tauri
-   **Frontend:** HTML5, CSS3, JavaScript (ES6)
-   **Styling:** Custom CSS to match the screenshots. No external CSS frameworks will be used initially to maintain a lean footprint.
-   **Icons:** SVG icons will be used for clarity and scalability.

## 4. Development Steps

1.  **HTML Scaffolding:** Create the basic HTML structure for `index.html` and `settings.html`.
2.  **Styling (CSS):** Implement the dark theme and layout for all views, focusing on matching the visual fidelity of the screenshots.
3.  **JavaScript Interactivity:**
    -   Implement tab switching for the Preferences window.
    -   Add basic client-side validation (e.g., password match).
4.  **Tauri Integration:**
    -   Define Tauri commands in the Rust backend for compression, extraction, and getting/setting preferences.
    -   Invoke these commands from the JavaScript frontend.
    -   Implement file drop handling using Tauri's event system.
