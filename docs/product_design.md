### 🎨 Vibe Mate - Product Design

**Project Context:**
Design the User Interface for "Vibe Mate," a desktop companion app for the "Vibe Coding" workflow. It manages AI proxies and Coding Agents (like Claude Code).
**Design Language:**

* **Theme:** "Cyberpunk Minimalist." Deep dark background (`#09090b`), high-contrast text, subtle neon purple/blue accents for active states.
* **Component System:** Shadcn/UI (Clean, accessible, radius-md).
* **Typography:** Monospace for config values (JetBrains Mono/Geist Mono), Sans-serif for UI text (Inter).

### 1. Global Layout (Navigation)

**Structure:** 2-Column Layout (Fixed Sidebar + Scrollable Content).

* **Sidebar (Left, 240px, Border-Right):**
* **Header:** App Icon + "Vibe Mate" (Bold).
* **Menu Items:**
1. General (Icon: `Settings`)
2. Model Provider (Icon: `Server`)
3. **Model Router** (Icon: `GitMerge` or `Route` - *Highlight this as a key feature*)
4. Coding Agents (Icon: `Bot`)
5. Network (Icon: `Globe`)


* **Footer:** System Status Dot (Green = Proxy Running) + Version info.

### 2. Feature: Model Provider Management

**Goal:** Manage connection credentials. Only one provider can be the "Default" system-wide.

**UI Specifications:**

* **Layout:** Grid of **Provider Cards**.
* **Card Anatomy:**
* **Header:** Provider Logo (e.g., OpenAI, Anthropic) + Name.
* **Body:**
* API Base url
* API Key(masked dots)
* Status Indicator.
* "Enable Proxy" Switch (controls if this provider uses the proxy).


* **Footer (Critical Interaction):**
* **"Default Provider" Toggle:**
* **Interaction:** Radio-button behavior. Only **one** card can be active at a time.
* **Active State:** Switch is ON, card border glows faintly, label says "System Default".
* **Inactive State:** Switch is OFF, label says "Set as Default".


### 3. Feature: Model Router

**Goal:** Visually configure how specific model names map to providers. This should look like a "Traffic Control" or "Firewall Rules" interface.

**UI Specifications:**

* **Visual Metaphor:** A vertical pipeline processing requests from top to bottom.
* **Section A: The Rules List (Sortable):**
* **List Item Component (Row):**
1. **Drag Handle:** (::) Icon on the far left. Cursor changes to `grab`.
2. **Match Pattern:** Input field. Placeholder: `gpt-4*` or `claude-3-5-*`. (Monospace font).
3. **Visual Flow:** An arrow icon (`ArrowRight`) pointing right.
4. **Target Provider:** Select/Dropdown. Options: [OpenAI, Anthropic, Gemini...].
5. **Model Rewrite (Optional):** Input field. Label: "Rewrite as...". Placeholder: `Leave empty to keep original`.
6. **Action:** Delete button (Trash icon, subtle).


* **Interaction:** Rows can be dragged to reorder priority.
* **Add Action:** A full-width dashed button at the bottom of the list: `+ Add Routing Rule`.


* **Section B: The "Catch-All" Fallback (Fixed Footer):**
* **Visual Style:** Distinct background (slightly lighter/darker), separated by a solid divider. Locked (cannot be dragged).
* **Content:**
* Icon: `ShieldAlert` or `CornerDownRight`.
* Text: **"Else (No match found)"**
* Visual Flow: `ArrowRight`
* Target: **Badge/Card** displaying the current **Default Provider** (e.g., "OpenAI").
* Helper Text: "Change the default provider in the 'Model Provider' tab."


### 4. Feature: Coding Agents

**Goal:** Monitoring and basic auth for CLI tools.

**UI Specifications:**

* **Layout:** Dashboard style.
* **Components:**
* **Agent Row/Card:**
* **Identity:** "Claude Code" or "Gemini CLI".
* **Connection Status:**
* *Authenticated:* Green Badge + "Logged In".
* *Disconnected:* Red Badge + "Login Required" Button.


* **Quota Usage:** A progress bar (e.g., "Daily Token Usage: 45%").
* **Config:** A "Settings" button (gear icon) opening a Sheet/Modal for environment variables.


### 5. Feature: Network

**Goal:** Proxy settings.

**UI Specifications:**

* **Layout:** Simple Form in a Card.
* **Fields:**
* Proxy Mode (Select: System / Custom / None).
* Host & Port (Inputs).
* **Action:** "Test Latency" Button.
* **Feedback:** When clicked, show a small toast or label: "🟢 120ms" or "🔴 Timeout".


### 💡 Design Notes for the AI

* **Interaction Logic:** When a user drags a rule in **Model Router**, add a smooth animation.
* **Empty States:** If no routing rules exist, show an illustration suggesting "All traffic will go to the Default Provider".
* **Micro-interactions:** Hovering over a Provider Card should slightly lift it (shadow-lg).