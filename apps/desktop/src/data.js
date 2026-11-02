import { DEFAULT_PERMISSIONS, validCapturedPermissions } from "./permissions.js";

// Sample data for the Software Factory design preview. Times are minutes since
// midnight on a fixed sample clock so the preview renders deterministically.

export const NOW = 12 * 60;
export const SCREENSHOT = `${import.meta.env?.BASE_URL ?? "/"}assets/greenroom-collection.png`;

export const clock = (m) => typeof m === "string" || m > 1e9
  ? new Date(m).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" })
  : `${Math.floor(m / 60)}:${String(m % 60).padStart(2, "0")}`;
export const dur = (mins) => {
  if (mins < 60) return `${Math.max(0, mins)}m`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m ? `${h}h ${m}m` : `${h}h`;
};

export const GOAL = {
  title: "Make the Collection faster to browse",
  focus: "Records carousel respects Reduce Motion",
};

export const MODELS = [
  { id: "codex-gpt-5.3", provider: "Codex", name: "GPT-5.3", efforts: ["low", "medium", "high"], available: true },
  { id: "claude-opus-5.5", provider: "Claude", name: "Opus 5.5", efforts: ["low", "medium", "high", "max"], available: true },
  { id: "gemini-3.1", provider: "Gemini", name: "3.1", efforts: ["medium"], available: false, note: "not connected" },
];
export const EFFORTS = [
  { id: "low", label: "Low" },
  { id: "medium", label: "Medium" },
  { id: "high", label: "High" },
  { id: "max", label: "Max" },
];
export const MODES = [
  { id: "auto", label: "Auto" },
  { id: "ask", label: "Ask first" },
  { id: "plan", label: "Plan only" },
];
export const ACCESS = [
  { id: "read-only", label: "Read only" },
  { id: "worktree", label: "Worktree edits" },
];

export const model = (id) => MODELS.find((m) => m.id === id);
export const modelLabel = (id) => {
  const m = model(id);
  return m ? `${m.provider} ${m.name}` : id;
};
export const effortLabel = (id) => EFFORTS.find((e) => e.id === id)?.label ?? id;
export const modeLabel = (id) => MODES.find((e) => e.id === id)?.label ?? id;
export const accessLabel = (id) => ACCESS.find((e) => e.id === id)?.label ?? id;

export function validateCombo(modelId, effortId) {
  const m = model(modelId);
  if (!m) return "Choose a model.";
  if (!m.available) return `${modelLabel(modelId)} is ${m.note} in this preview. Choose another model.`;
  if (!m.efforts.includes(effortId)) {
    const ok = m.efforts.map(effortLabel).join(", ");
    return `${modelLabel(modelId)} doesn't support ${effortLabel(effortId)} effort. Choose ${ok}.`;
  }
  return null;
}

export const SOURCE_LABEL = {
  default: "Project default",
  override: "Next-run override",
  copied: "Copied from earlier attempt",
  edit: "Retry edit",
};

export const ROOT_OPTIONS = [
  { id: "/", label: "Project root files" },
  { id: "src/", label: "src/" },
  { id: "docs/", label: "docs/" },
  { id: ".factory/", label: ".factory/" },
];

export const DEFAULT_SETTINGS = {
  projectName: "Green Room",
  baseBranch: "main",
  model: "codex-gpt-5.3",
  effort: "medium",
  mode: "ask",
  concurrency: 2,
  budget: 40,
  access: "worktree",
  permissions: DEFAULT_PERMISSIONS,
  roots: ["/", "src/", ".factory/"],
  ignore: "DerivedData/\n*.xcuserstate\n.env*",
  alwaysAgents: true,
  excludedFiles: [],
};

export const STATUS = {
  claimed: "Claimed",
  checking: "Checking",
  recovery_required: "Recovery required",
  queued: "Queued",
  running: "Running",
  interrupting: "Interrupting…",
  needs_approval: "Needs approval",
  succeeded: "Succeeded",
  failed: "Failed",
  interrupted: "Interrupted",
  cancelled: "Cancelled",
};
export const ACTIVE = ["queued", "claimed", "running", "checking", "interrupting", "needs_approval", "recovery_required"];
export const TERMINAL_RETRY = ["failed", "interrupted", "cancelled"];
export const FILTERS = [
  { id: "all", label: "All", match: () => true },
  { id: "running", label: "Running", match: (s) => ["queued", "claimed", "running", "checking", "interrupting"].includes(s) },
  { id: "waiting", label: "Waiting", match: (s) => ["needs_approval", "recovery_required"].includes(s) },
  { id: "finished", label: "Finished", match: (s) => ["succeeded", "failed", "interrupted", "cancelled"].includes(s) },
];

export const TASKS = [
  { id: "T-carousel", title: "Records carousel respects Reduce Motion", short: "Carousel motion" },
  { id: "T-maplink", title: "Venue detail map link", short: "Map link" },
  { id: "T-media", title: "Refresh media cache", short: "Media cache" },
  { id: "T-grid", title: "Tighten Collection grid spacing", short: "Grid spacing" },
  { id: "T-search", title: "Venue search empty state", short: "Search empty state" },
  { id: "T-share", title: "Share sheet button label", short: "Share label" },
  { id: "T-sort", title: "Artist sort ignores articles", short: "Artist sort" },
  { id: "T-ticket", title: "Ticket stub export", short: "Ticket export" },
  { id: "T-filters", title: "Discover filters persist", short: "Discover filters" },
];

const CTX = ["README.md", "AGENTS.md"];
const snap = (m, effort, mode, source, extra = {}) => ({
  model: m,
  effort,
  mode,
  source,
  access: "worktree",
  contextRevision: "b93d0f1",
  contextFiles: CTX,
  ...extra,
});
const all = (s) => ({ model: s, effort: s, mode: s });

export const RUNS = [
  {
    id: "R-8K4D", taskId: "T-carousel", attempt: 2, status: "running",
    startedAt: 706, updatedAt: 719, endedAt: null, revision: "d27b4f8", heartbeatAt: 719,
    stages: [{ stage: "build", at: 706 }], retriedFrom: "R-7Q2M",
    settings: snap("codex-gpt-5.3", "high", "ask", { model: "edit", effort: "copied", mode: "copied" },
      { contextFiles: [...CTX, "src/Collection/RecordsCarousel.swift"], capturedAt: 706 }),
    activity: [
      { at: 706, text: "Started from Attempt 1's failing UI test. Reading the carousel timer.", tools: [
        "Read src/Collection/RecordsCarousel.swift", "Search \"autoAdvance\" · 3 matches"] },
      { at: 713, text: "Gated auto-advance on accessibilityReduceMotion and kept manual swipes.", tools: [
        "Edit src/Collection/RecordsCarousel.swift · +14 −6"] },
      { at: 718, text: "Running UI tests on d27b4f8.", tools: ["Command xcodebuild test -only-testing RecordsCarouselTests"] },
    ],
    changed: [{ path: "src/Collection/RecordsCarousel.swift", add: 14, del: 6 }],
    checks: [
      { name: "UI tests", result: "running", at: 718, revision: "d27b4f8" },
      { name: "Build", result: "pass", at: 640, revision: "5c1a9e0" },
    ],
    artifacts: [
      { label: "Carousel screen recording", kind: "recording", state: "missing", revision: "d27b4f8", note: "Recorded after checks finish" },
    ],
  },
  {
    id: "R-7Q2M", taskId: "T-carousel", attempt: 1, status: "failed",
    startedAt: 605, updatedAt: 658, endedAt: 658, revision: "5c1a9e0",
    stages: [{ stage: "build", at: 605 }, { stage: "check", at: 636 }],
    failure: "UI test RecordsCarouselTests.testReduceMotion failed: the carousel still auto-advanced with Reduce Motion on.",
    settings: snap("claude-opus-5.5", "high", "auto", { model: "override", effort: "override", mode: "default" }, { capturedAt: 605 }),
    activity: [
      { at: 605, text: "Planned a motion check for the records carousel.", tools: ["Read src/Collection/RecordsCarousel.swift"] },
      { at: 622, text: "Slowed the transition instead of stopping auto-advance.", tools: ["Edit src/Collection/RecordsCarousel.swift · +5 −2"] },
      { at: 636, text: "UI tests ran on 5c1a9e0; one failed.", tools: ["Command xcodebuild test · 1 failed, 31 passed"] },
    ],
    changed: [{ path: "src/Collection/RecordsCarousel.swift", add: 5, del: 2 }],
    checks: [
      { name: "Build", result: "pass", at: 640, revision: "5c1a9e0" },
      { name: "UI tests", result: "fail", at: 657, revision: "5c1a9e0" },
    ],
    artifacts: [
      { label: "Failing test report", kind: "report", state: "current", revision: "5c1a9e0", note: "Inline summary only in this preview" },
      { label: "Screen recording", kind: "recording", state: "expired", revision: "5c1a9e0", note: "Expired after 24 hours" },
    ],
  },
  {
    id: "R-6H1C", taskId: "T-maplink", attempt: 1, status: "running",
    startedAt: 680, updatedAt: 719, endedAt: null, revision: "e4b7c21", heartbeatAt: 719,
    stages: [{ stage: "build", at: 680 }, { stage: "check", at: 711 }],
    settings: snap("claude-opus-5.5", "medium", "ask", { model: "override", effort: "default", mode: "default" }, { capturedAt: 680 }),
    activity: [
      { at: 680, text: "Added an Open in Maps row to venue detail.", tools: ["Read src/Venue/VenueDetailView.swift", "Edit src/Venue/VenueDetailView.swift · +22 −1"] },
      { at: 711, text: "Supervisor check started on e4b7c21.", tools: ["Command xcodebuild test -only-testing VenueDetailTests"] },
    ],
    changed: [{ path: "src/Venue/VenueDetailView.swift", add: 22, del: 1 }],
    checks: [{ name: "Venue detail tests", result: "running", at: 711, revision: "e4b7c21" }, { name: "Accessibility audit", result: "pending", at: null, revision: "e4b7c21" }],
    artifacts: [{ label: "Venue detail screenshot", kind: "screenshot", state: "missing", revision: "e4b7c21", note: "Captured after checks" }],
  },
  {
    id: "R-5N8P", taskId: "T-media", attempt: 1, status: "needs_approval",
    startedAt: 632, updatedAt: 718, endedAt: null, revision: "9a0f3d6", heartbeatAt: 718,
    stages: [{ stage: "build", at: 632 }, { stage: "waiting", at: 650 }],
    approval: { request: "Network access to media.greenroom.example", scope: "This run only", requestedAt: 650, short: "network approval" },
    settings: snap("codex-gpt-5.3", "medium", "ask", all("default"), { capturedAt: 632 }),
    activity: [
      { at: 632, text: "Found the stale cache key and drafted a refresh step.", tools: ["Read src/Media/MediaCache.swift"] },
      { at: 650, text: "Paused: refreshing needs network access. Waiting for your decision.", tools: ["Request network · media.greenroom.example"] },
    ],
    changed: [],
    checks: [],
    artifacts: [],
  },
  {
    id: "R-3B7T", taskId: "T-grid", attempt: 1, status: "succeeded",
    startedAt: 612, updatedAt: 663, endedAt: 663, revision: "a4f3c2e", reviewId: "RV-24",
    stages: [{ stage: "build", at: 612 }, { stage: "check", at: 648 }],
    settings: snap("codex-gpt-5.3", "medium", "ask", all("default"), { capturedAt: 612 }),
    activity: [
      { at: 612, text: "Read the grid and card layout.", tools: ["Search \"spacing\" src/Collection · 6 matches", "Read src/Collection/ArtistRecordCard.swift"] },
      { at: 631, text: "Reduced gutters from 20 to 12 pt and aligned show counts to the name baseline.", tools: ["Edit src/Collection/ArtistRecordCard.swift · +18 −11", "Edit src/Collection/CollectionGrid.swift · +4 −4"] },
      { at: 648, text: "Checks passed on a4f3c2e. Opened review #24.", tools: ["Command xcodebuild test · 48 passed"] },
    ],
    changed: [
      { path: "src/Collection/ArtistRecordCard.swift", add: 18, del: 11 },
      { path: "src/Collection/CollectionGrid.swift", add: 4, del: 4 },
      { path: "Tests/CollectionGridSnapshotTests.swift", add: 6, del: 2 },
    ],
    checks: [
      { name: "Build", result: "pass", at: 650, revision: "a4f3c2e" },
      { name: "Unit tests", result: "pass", at: 654, revision: "a4f3c2e" },
      { name: "Snapshot tests", result: "pass", at: 656, revision: "a4f3c2e" },
      { name: "SwiftLint", result: "pass", at: 657, revision: "a4f3c2e" },
      { name: "Accessibility audit", result: "pass", at: 660, revision: "a4f3c2e" },
      { name: "Dynamic Type layout", result: "pass", at: 662, revision: "a4f3c2e" },
    ],
    artifacts: [
      { label: "Collection screenshot", kind: "screenshot", state: "current", revision: "a4f3c2e", href: SCREENSHOT },
      { label: "Accessibility-size screenshot", kind: "screenshot", state: "missing", revision: "a4f3c2e", note: "Couldn't be captured" },
      { label: "Grid screenshot (before)", kind: "screenshot", state: "stale", revision: "7d21e0b", href: SCREENSHOT, note: "Older revision" },
    ],
  },
  {
    id: "R-4C2V", taskId: "T-search", attempt: 1, status: "succeeded",
    startedAt: 560, updatedAt: 638, endedAt: 638, revision: "71d09be", reviewId: "RV-23",
    stages: [{ stage: "build", at: 560 }, { stage: "check", at: 620 }],
    settings: snap("claude-opus-5.5", "high", "ask", { model: "override", effort: "override", mode: "default" }, { capturedAt: 560 }),
    activity: [
      { at: 560, text: "Added an empty state for venue search with a clear-search action.", tools: ["Edit src/Venue/VenueSearchView.swift · +16 −3"] },
      { at: 620, text: "Checks passed on 71d09be. Opened review #23.", tools: ["Command xcodebuild test · 22 passed"] },
    ],
    changed: [{ path: "src/Venue/VenueSearchView.swift", add: 16, del: 3 }],
    checks: ["Build", "Unit tests", "UI tests", "SwiftLint", "Accessibility audit"].map((name, i) => ({ name, result: "pass", at: 624 + i * 3, revision: "71d09be" })),
    artifacts: [{ label: "Empty state screenshot", kind: "screenshot", state: "missing", revision: "71d09be", note: "Not captured" }],
  },
  {
    id: "R-2F9A", taskId: "T-share", attempt: 1, status: "succeeded",
    startedAt: 530, updatedAt: 571, endedAt: 571, revision: "0e5d7aa", reviewId: "RV-22",
    stages: [{ stage: "build", at: 530 }, { stage: "check", at: 560 }],
    settings: snap("codex-gpt-5.3", "low", "auto", { model: "default", effort: "override", mode: "override" }, { capturedAt: 530 }),
    activity: [{ at: 530, text: "Renamed the share sheet button and updated its accessibility label.", tools: ["Edit src/Show/ShareShowButton.swift · +3 −3"] }],
    changed: [{ path: "src/Show/ShareShowButton.swift", add: 3, del: 3 }],
    checks: [{ name: "Build", result: "pass", at: 566, revision: "0e5d7aa" }, { name: "Unit tests", result: "pass", at: 569, revision: "0e5d7aa" }],
    artifacts: [],
  },
  {
    id: "R-9P5L", taskId: "T-sort", attempt: 1, status: "failed",
    startedAt: 668, updatedAt: 692, endedAt: 692, revision: "3e8b1f4",
    stages: [{ stage: "build", at: 668 }],
    failure: "Build failed: ArtistSort.swift — ambiguous use of 'localizedStandardCompare'.",
    settings: snap("codex-gpt-5.3", "medium", "ask", all("default"), { capturedAt: 668 }),
    activity: [
      { at: 668, text: "Sorted artists ignoring leading “The”.", tools: ["Edit src/Collection/ArtistSort.swift · +9 −2"] },
      { at: 690, text: "Build failed on 3e8b1f4.", tools: ["Command xcodebuild build · 1 error"] },
    ],
    changed: [{ path: "src/Collection/ArtistSort.swift", add: 9, del: 2 }],
    checks: [{ name: "Build", result: "fail", at: 691, revision: "3e8b1f4" }],
    artifacts: [{ label: "Build log", kind: "report", state: "current", revision: "3e8b1f4", note: "Inline summary only in this preview" }],
  },
  {
    id: "R-1D3E", taskId: "T-ticket", attempt: 1, status: "interrupted",
    startedAt: 580, updatedAt: 615, endedAt: 615, revision: "c0a4e92",
    stages: [{ stage: "plan", at: 580 }],
    failure: "Interrupted by you at 10:15 before any change was made.",
    settings: snap("claude-opus-5.5", "medium", "plan", { model: "override", effort: "default", mode: "override" }, { capturedAt: 580 }),
    activity: [{ at: 580, text: "Outlined export formats for ticket stubs.", tools: ["Read README.md"] }, { at: 615, text: "Interrupted before any edits." }],
    changed: [],
    checks: [{ name: "Build", result: "unknown", at: null, revision: "c0a4e92" }],
    artifacts: [],
  },
];

export const LEAD_MESSAGES = [
  { id: "m1", role: "you", at: 582, text: "This week, focus on Collection. The grid feels loose and the carousel ignores Reduce Motion." },
  { id: "m2", role: "lead", at: 584, text: "Saved the goal “Make the Collection faster to browse” and split it into tasks. Codex is on the carousel; Claude is adding the map link on venue detail.", link: "progress" },
  { id: "m3", role: "lead", at: 665, text: "Grid spacing is ready for review. Checks passed on a4f3c2e. I couldn't capture an accessibility-size screenshot, so that's listed as a limitation.", review: "RV-24" },
  { id: "m4", role: "lead", at: 650, text: "Refreshing the media cache needs network access. Allow it for this run only?", approval: "R-5N8P" },
].sort((a, b) => a.at - b.at);

const D = (s) =>
  s.replace(/^\n/, "").split("\n").map((l) => ({ t: l[0] === "+" ? "add" : l[0] === "-" ? "del" : "ctx", s: l.slice(2) }));

export const REVIEWS = [
  {
    id: "RV-24", number: 24, runId: "R-3B7T", attempt: 1, revision: "a4f3c2e", title: "Tighten Collection grid spacing", group: "ready",
    summary: "Reduces grid card gutters from 20 to 12 points and aligns show counts to the artist name baseline.",
    limitation: "Couldn't capture an accessibility-size screenshot. The Dynamic Type layout check passed in tests only.",
    preview: { src: SCREENSHOT, alt: "Green Room Collection screen with a two-column artist grid" },
    files: [
      { path: "src/Collection/ArtistRecordCard.swift", add: 18, del: 11, lines: D(`
  import SwiftUI
  
  struct ArtistRecordCard: View {
      let artist: Artist
      let showCount: Int
  
      var body: some View {
          VStack(alignment: .leading, spacing: 12) {
              HStack(spacing: 12) {
                  AlbumArtView(artist: artist)
                  RecordView(artist: artist)
              }
-             .padding(.horizontal, 20)
+             .padding(.horizontal, 12)
  
              VStack(alignment: .leading, spacing: 4) {
                  Text(artist.name)
                      .font(.system(size: 16, weight: .medium))
+                     .alignmentGuide(.firstTextBaseline) { d in
+                         d[.firstTextBaseline]
+                     }
                  Text("\\(showCount) show\\(showCount == 1 ? "" : "s")")
                      .font(.system(size: 13))
                      .foregroundStyle(.secondary)
              }
          }
      }
  }
  
  struct CollectionGrid: View {
      let artists: [Artist]
  
      var body: some View {
          LazyVGrid(
-             columns: [GridItem(.flexible())],
-             spacing: 20
+             columns: [GridItem(.flexible()), GridItem(.flexible())],
+             spacing: 12
          ) {
              ForEach(artists) { ArtistRecordCard(artist: $0, showCount: $0.shows) }
          }
      }
  }`) },
      { path: "src/Collection/CollectionGrid.swift", add: 4, del: 4, lines: D(`
  extension CollectionGrid {
-     static let gutter: CGFloat = 20
-     static let rowSpacing: CGFloat = 24
+     static let gutter: CGFloat = 12
+     static let rowSpacing: CGFloat = 16
  
      var columns: [GridItem] {
-         [GridItem(.adaptive(minimum: 160), spacing: Self.gutter)]
-         .map { $0 }
+         Array(repeating: GridItem(.flexible(), spacing: Self.gutter), count: 2)
+         // two columns at every Dynamic Type size up to XXL
      }
  }`) },
      { path: "Tests/CollectionGridSnapshotTests.swift", add: 6, del: 2, lines: D(`
  final class CollectionGridSnapshotTests: XCTestCase {
      func testTwoColumnGrid() {
-         let view = CollectionGrid(artists: .sample)
-         assertSnapshot(of: view, as: .image)
+         let view = CollectionGrid(artists: .sample)
+         assertSnapshot(of: view, as: .image(layout: .device(config: .iPhone15)))
+     }
+
+     func testLargeTextKeepsColumns() {
+         assertSnapshot(of: CollectionGrid(artists: .sample).dynamicTypeSize(.xxLarge), as: .image)
      }
  }`) },
    ],
  },
  {
    id: "RV-23", number: 23, runId: "R-4C2V", attempt: 1, revision: "71d09be", title: "Venue search empty state", group: "ready",
    summary: "Shows a clear empty state when venue search has no results, with a one-tap way to clear the query.",
    limitation: null,
    preview: null,
    previewNote: "No screenshot was captured for this revision.",
    files: [
      { path: "src/Venue/VenueSearchView.swift", add: 16, del: 3, lines: D(`
  struct VenueSearchView: View {
      @State private var query = ""
      let venues: [Venue]
  
      var body: some View {
          List(results) { VenueRow(venue: $0) }
-             .overlay {
-                 if results.isEmpty { ProgressView() }
-             }
+             .overlay {
+                 if results.isEmpty && !query.isEmpty {
+                     ContentUnavailableView {
+                         Label("No venues match “\\(query)”", systemImage: "magnifyingglass")
+                     } actions: {
+                         Button("Clear search") { query = "" }
+                     }
+                 }
+             }
              .searchable(text: $query)
      }
  }`) },
    ],
  },
  {
    id: "RV-22", number: 22, runId: "R-2F9A", attempt: 1, revision: "0e5d7aa", title: "Share sheet button label", group: "changes",
    summary: "Renames the share sheet button from “Send” to “Share show” and updates its accessibility label.",
    limitation: null,
    requested: "Use “Share” on its own; “Share show” wraps on smaller phones.",
    preview: null,
    previewNote: "Text-only change. No preview for this revision.",
    files: [
      { path: "src/Show/ShareShowButton.swift", add: 3, del: 3, lines: D(`
  struct ShareShowButton: View {
      let show: Show
  
      var body: some View {
          ShareLink(item: show.url) {
-             Label("Send", systemImage: "paperplane")
+             Label("Share show", systemImage: "square.and.arrow.up")
          }
-         .accessibilityLabel("Send")
+         .accessibilityLabel("Share \\(show.title)")
-         .font(.callout)
+         .font(.body)
      }
  }`) },
    ],
  },
];
export const MERGED = ["#21 Show detail dates", "#20 Artist search debounce", "#19 Log button hit area"];

export const FILES = {
  "README.md": `# Green Room

A personal concert diary for iOS. Log shows, collect artists and venues,
and browse your history as a record collection.

## Develop
- Xcode 17, iOS 19 SDK
- \`make test\` runs unit and snapshot tests
`,
  "AGENTS.md": `# Agent notes

- Keep SwiftUI views small; prefer composition over modifiers chains.
- Respect Reduce Motion and Dynamic Type in every change.
- Never commit credentials. Media hosts need explicit approval.
- Run \`make test\` before asking for review.
`,
  ".factory/config.toml": `# Sample project file. Editing it here does not change Settings.
[project]
name = "Green Room"
base_branch = "main"

[defaults]
model = "codex-gpt-5.3"
effort = "medium"
mode = "ask"

[context]
roots = ["/", "src/", ".factory/"]
ignore = ["DerivedData/", "*.xcuserstate", ".env*"]
`,
  "docs/collection.md": `# Collection

The Collection tab groups artists, venues and shows.
Artists render as album art with a record peeking out.
`,
  "src/Collection/CollectionGrid.swift": `import SwiftUI

struct CollectionGrid: View {
    let artists: [Artist]

    var body: some View {
        LazyVGrid(columns: columns, spacing: Self.rowSpacing) {
            ForEach(artists) { ArtistRecordCard(artist: $0, showCount: $0.shows) }
        }
        .padding(.horizontal, 16)
    }
}
`,
  "src/Collection/ArtistRecordCard.swift": `import SwiftUI

struct ArtistRecordCard: View {
    let artist: Artist
    let showCount: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 12) {
                AlbumArtView(artist: artist)
                RecordView(artist: artist)
            }
            Text(artist.name).font(.system(size: 16, weight: .medium))
        }
    }
}
`,
  "src/Collection/RecordsCarousel.swift": `import SwiftUI

struct RecordsCarousel: View {
    @Environment(\\.accessibilityReduceMotion) private var reduceMotion
    let records: [Record]
    @State private var index = 0

    var body: some View {
        TabView(selection: $index) {
            ForEach(records.indices, id: \\.self) { RecordView(record: records[$0]).tag($0) }
        }
        .tabViewStyle(.page)
        .onReceive(timer) { _ in
            guard !reduceMotion else { return }
            withAnimation { index = (index + 1) % records.count }
        }
    }
}
`,
  "src/Venue/VenueDetailView.swift": `import SwiftUI
import MapKit

struct VenueDetailView: View {
    let venue: Venue

    var body: some View {
        List {
            Section { Text(venue.address) }
            Section("Shows") { ForEach(venue.shows) { ShowRow(show: $0) } }
        }
        .navigationTitle(venue.name)
    }
}
`,
  "src/Media/MediaCache.swift": `import Foundation

actor MediaCache {
    private var entries: [URL: Data] = [:]
    func data(for url: URL) -> Data? { entries[url] }
    func store(_ data: Data, for url: URL) { entries[url] = data }
}
`,
};

export function rootOf(path) {
  const i = path.indexOf("/");
  return i === -1 ? "/" : path.slice(0, i + 1);
}

function matchesIgnore(path, patterns) {
  return patterns.some((p) => {
    if (!p) return false;
    const directory = p.endsWith("/");
    const glob = [...(directory ? p.slice(0, -1) : p)].map((c) =>
      c === "*" ? "[^/]*" : c === "?" ? "[^/]" : c.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"),
    ).join("");
    const re = new RegExp(`(^|/)${glob}${directory ? "/" : "$"}`);
    return re.test(path);
  });
}

export function contextStatus(path, settings) {
  if (path === "AGENTS.md" && settings.alwaysAgents) return { included: true, reason: "Always included" };
  if (settings.excludedFiles.includes(path)) return { included: false, reason: "Excluded by you" };
  const root = rootOf(path);
  if (!settings.roots.includes(root)) return { included: false, reason: `Root ${root === "/" ? "project files" : root} is off` };
  const patterns = settings.ignore.split("\n").map((s) => s.trim()).filter(Boolean);
  if (matchesIgnore(path, patterns)) return { included: false, reason: "Matches an ignore pattern" };
  return { included: true, reason: root === "/" ? "Project root file" : `Under ${root}` };
}

export function captureContext(settings, attachments = [], files = FILES) {
  return {
    roots: [...settings.roots],
    ignore: settings.ignore,
    excludedFiles: [...settings.excludedFiles],
    alwaysAgents: settings.alwaysAgents,
    contextFiles: [...new Set([
      ...Object.keys(files).filter((path) => contextStatus(path, settings).included),
      ...attachments,
    ])],
  };
}

export function restoreRunHistory(saved) {
  return (Array.isArray(saved) ? saved : RUNS).filter((run) => run && validCapturedPermissions(run.settings?.permissions)).map((run) => run.status === "interrupting" ? {
    ...run, status: "interrupted", endedAt: NOW, updatedAt: NOW,
    failure: "Interruption finished locally when this preview reloaded. No real process was stopped.",
    activity: [...run.activity, { at: NOW, text: "Preview interruption completed after reload." }],
  } : run);
}

export function taskState(task, runs, reviewStatus) {
  if (task.live) return ({ ready: "Ready", running: "Running", review: "Review", blocked: "Blocked", done: "Done" })[task.status] ?? "Blocked";
  const rs = runs.filter((r) => r.taskId === task.id).sort((a, b) => a.attempt - b.attempt);
  if (!rs.length) return "Backlog";
  if (rs.some((r) => r.status === "needs_approval")) return "Blocked";
  if (rs.some((r) => r.status === "running" || r.status === "interrupting")) return "Running";
  const last = rs[rs.length - 1];
  if (last.status === "queued") return "Ready";
  if (last.status === "succeeded") return reviewStatus[last.reviewId] === "merged" ? "Done" : "Review";
  return "Ready";
}

export function primaryRun(taskId, runs) {
  const rs = runs.filter((r) => r.taskId === taskId).sort((a, b) => b.attempt - a.attempt);
  return rs.find((r) => ACTIVE.includes(r.status)) ?? rs[0] ?? null;
}

export function hashId(s) {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619);
  return (h >>> 0) / 4294967295;
}

export function stageOf(run) {
  if (run.status === "needs_approval") return { stage: "waiting", since: run.approval?.requestedAt ?? run.updatedAt ?? run.startedAt };
  const s = run.stages[run.stages.length - 1];
  return { stage: s.stage, since: s.at };
}
