#!/usr/bin/env python3
"""Generate docs/FEATURE-PARITY.csv and fail if any inventoried symbol is unaccounted for.

Run from anywhere: python -I docs/reference/audit/build_parity.py
Inputs are only the two inventories and the derived reach/IPC JSON in this folder.
This is an audit tool, not product code.
"""
import csv
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
DOCS = os.path.dirname(os.path.dirname(HERE))
MF = json.load(open(os.path.join(HERE, "..", "myfranchise-2.0.1-inventory.json"), encoding="utf8"))
DG = json.load(open(os.path.join(HERE, "..", "draft-tool-inventory.json"), encoding="utf8"))
REACH = json.load(open(os.path.join(HERE, "mf201-component-reach.json"), encoding="utf8"))
CALLS = json.load(open(os.path.join(HERE, "mf201-ipc-callers.json"), encoding="utf8"))

COLUMNS = [
    "id", "row_type", "source", "source_version", "family", "screen_or_operation", "evidence",
    "observable_behavior", "m27_applicability", "dependencies", "rust_owner", "ui_owner",
    "acceptance_test", "runtime_evidence", "status", "work_package", "pr_link",
    "symbol_kind", "symbol", "classification", "maps_to",
]

MF_V = "MyFranchise 2.0.1"
DG_V = "draft generator 1.5.2 (04b24393 + dirty tree)"
R_MF = "static-only: not run; needs WP00R golden capture and in-game check where it writes"
R_DG = "code and tests read, not executed in this audit; legacy 1.5.0 report records 366 server tests passing"
R_DG_F = "code read only; legacy routes are gated off by DRAFT_TOOL_FRANCHISE; M27 behaviour unproven"
R_X = "new requirement: no oracle; proof comes from implementation gates"

rows = []
ids = set()


def add(fid, source, version, family, op, evidence, behavior, deps, rust, ui, test, wp,
        runtime, app="applicable", status="discovered"):
    assert fid not in ids, fid
    ids.add(fid)
    rows.append(dict(
        id=fid, row_type="feature", source=source, source_version=version, family=family,
        screen_or_operation=op, evidence=evidence, observable_behavior=behavior,
        m27_applicability=app, dependencies=deps, rust_owner=rust, ui_owner=ui,
        acceptance_test=test, runtime_evidence=runtime, status=status, work_package=wp,
        pr_link="", symbol_kind="", symbol="", classification="feature", maps_to=""))


def M(fid, family, op, ev, beh, deps, rust, ui, test, wp, app="applicable"):
    add(fid, "MyFranchise", MF_V, family, op, ev, beh, deps, rust, ui, test, wp, R_MF, app)


def D(fid, family, op, ev, beh, deps, rust, ui, test, wp, app="applicable", runtime=R_DG, status="discovered"):
    add(fid, "Draft generator", DG_V, family, op, ev, beh, deps, rust, ui, test, wp, runtime, app, status)


def X(fid, family, op, ev, beh, deps, rust, ui, test, wp):
    add(fid, "Binding requirement", "design + OPUS-MASTER", family, op, ev, beh, deps, rust, ui, test, wp, R_X)


# Rust owners
FF = "mt-franchise-format"
FO = "mt-franchise"
DF = "mt-draft-format"
RF = "mt-roster-format"
GEN = "mt-draftgen"
ST = "mt-store"
IO = "mt-interop"
CO = "mt-core"
TA = "app/src-tauri"
READ = "F-READ"
PREV = "edit -> change-set preview -> commit"

# -------------------------------------------------------------------------- cross-cutting
X("XP-001", "Platform", "Windows x64 build, install, run", "design: cross-platform", "App installs and runs on Windows x64.", "", TA, "ui/shell", "Install on a Windows runner and a real desktop; open a copied draft class and export it.", "11a")
X("XP-002", "Platform", "macOS arm64 and x64 build, install, run", "design: cross-platform", "App runs on both macOS architectures; unsigned state is labelled.", "", TA, "ui/shell", "Native runner build plus a smoke test on real or hosted macOS; record whether it is signed or notarized.", "11a")
X("XP-003", "Platform", "Linux x64 AppImage and deb", "design: cross-platform", "App runs on Linux with the required WebKitGTK.", "", TA, "ui/shell", "Install both packages on a clean runner under xvfb; open and export a copied class.", "11a")
X("XP-004", "Platform", "User-selected paths on every OS", "design; MF main.js saves lookup is Windows-shaped", "Every file operation works from a user-picked path; Saves discovery is only a convenience.", "XP-001;XP-002;XP-003", CO, "ui/files", "On all three OSes open and export with Saves discovery unavailable.", "01b")
X("XP-005", "Responsive UI", "Layouts at 800x600, 1024x768, 1440x900, 1920x1080, 200% zoom", "design; MF window is fixed, non-resizable (main.js BrowserWindow resizable:false)", "No clipped dialog or unreachable action at any listed size.", "", "", "ui/shell", "Automated viewport screenshots and an action-reachability check on every route.", "01b")
X("XP-006", "Responsive UI", "Virtualized large lists", "design; legacy VirtualList; ~32k pool", "Player, pool and stat tables scroll smoothly; data is paged by Rust.", "XP-009", CO, "ui/table", "Record p95 filter and sort under 200 ms on 32,140 rows with hardware and build mode.", "01b")
X("XP-007", "Responsive UI", "Keyboard, focus, screen-reader status, reduced motion", "design", "Every action is keyboard reachable with visible focus and announced status.", "", "", "ui/shell", "Axe scan plus a keyboard-only script per feature family.", "01b")
X("XP-008", "Reactive UI", "Revision-based invalidation", "design", "A commit returns revision and affected IDs; only dependent queries refetch; selection and scroll stay put.", "", CO, "ui/query", "Edit a player while a list is open; the row updates with no full reload and keeps its scroll position.", "01b")
X("XP-009", "Jobs", "Background job lifecycle", "design", "Jobs report queued, running, progress, cancelling, succeeded, failed or cancelled over a typed channel.", "", CO, "ui/jobs", "Cancel a long job at 50%; the source file hash is unchanged and no temp file remains.", "01b")
X("XP-010", "Jobs", "Stale-result protection", "design", "A result from an older job or revision never overwrites newer state.", "XP-009", CO, "ui/query", "Fire overlapping queries with injected delay; only the newest renders.", "01b")
X("XP-011", "Save safety", "Transactional commit", "design; MF saves in place (franchise:save) and legacy export writes tmp then rename", "Hash check, invariant check, sibling temp, flush, reopen-validate, atomic replace.", "XP-009", CO, "ui/files", "Fault injection at each step leaves the source byte-identical on all three OSes.", "02b")
X("XP-012", "Save safety", "External-change conflict", "MF franchise-file-changed; design", "A changed source blocks the commit and offers reload or save as a new file.", "XP-011", CO, "ui/files", "Modify the source between preview and commit; the commit refuses with a clear message.", "02b")
X("XP-013", "Save safety", "Disk-full, locked file and interrupted process", "design", "Failures are reported and the source stays intact; orphan temps are found on next launch.", "XP-011", CO, "ui/files", "Simulate ENOSPC and a Windows sharing violation; kill the process mid-commit.", "02b")
X("XP-014", "Format", "Refuse other games and unknown versions", "MF franchise:open rejects non-M27; legacy detectGame uses size only", "M26, CFB and unknown M27 schemas are refused for editing with a useful message.", "", CO, "ui/files", "M26 career save and CFB-schema input are refused; unknown schema major opens read-only or not at all.", "02a")
X("XP-015", "Licensing", "Third-party notices and asset provenance", "SOURCES.md; audit section 8", "Shipped material has a recorded right to ship; proprietary art and saves stay out of the repo.", "", "", "ui/about", "Release check fails on any file not listed in the provenance manifest.", "01a")
X("XP-016", "Delivery", "Parity ledger and delivery ledger gate", "design; roadmap", "Each applicable row reaches runtime-verified or carries a precise blocker.", "", "", "", "verify_parity.py passes and no applicable row is below runtime-verified at release.", "11c")

# -------------------------------------------------------------------------- MF: file and session
S_MAIN = "asar .vite/build/main.js"
S_R = "asar index-CqbtHSxo.js"
E = "edit -> preview -> commit"
M("MF-FILE-01", "File session", "List franchise saves with live metadata", S_MAIN + " fs:readMaddenSavesFolderBasic, fs:streamFranchiseMeta, franchiseMetaWorker.js; HomeView", "Load screen lists CAREER files from Documents or OneDrive with year, week, record, teams and dates, filled in progressively.", "", FO, "ui/files", "Folder of copied files: rows appear before metadata finishes; CAREERDRAFT, PROFILE, ROSTER and FRANCHISE files are excluded.", "05c")
M("MF-FILE-02", "File session", "Show or hide autosaves and rescan", S_R + " HomeView strings", "Toggle hides autosaves; refresh rescans and confirms with a toast.", "MF-FILE-01", FO, "ui/files", "Toggle and refresh change the list as expected.", "05c")
M("MF-FILE-03", "File session", "Open a file by picker", S_MAIN + " fs:openFileDialog; HomeView", "A native picker opens any path. MF reads it as utf8 only to return a path; MaddenTool opens by path only.", "XP-004", FO, "ui/files", "Open a copied save from a non-Saves path on all three OSes.", "05c")
M("MF-FILE-04", "File session", "Reject unsupported files", S_MAIN + " franchise:open", "Blocked prefixes, non-Madden and non-M27 files give a specific error.", "XP-014", FF, "ui/files", "Each rejected class shows its own message and changes nothing.", "05c")
M("MF-FILE-05", "File session", "Autosave warning", S_R + " dB().P() Loading Autosave File", "Opening *-AUTOSAVE asks for confirmation unless the setting is off.", "MF-SET-05", FO, "ui/files", "Prompt appears, can be cancelled, and is suppressed by the setting.", "05c")
M("MF-FILE-06", "File session", "External change prompt", S_MAIN + " chokidar franchise-file-changed; App component", "When the save changes outside the app, a prompt offers reload or dismiss.", "XP-012", FO, "ui/files", "Touch the copied file; prompt appears; own commits do not trigger it.", "05c")
M("MF-FILE-07", "File session", "Reload current file", S_MAIN + " franchise:reload; SettingsPanel; TeamNavbar", "Reloads from disk and resets navigation with a confirmation toast.", "MF-FILE-06", FO, "ui/files", "External edit becomes visible after reload; unsaved changes warn first.", "05c")
M("MF-FILE-08", "File session", "Close file", S_MAIN + " franchise:close, db:close; TeamNavbar", "Returns to the load screen and closes the watcher and database.", "", FO, "ui/files", "No handle is left open: the file can be renamed on Windows after close.", "05c")
M("MF-FILE-09", "File session", "Automatic backup on open with retention", S_MAIN + " fs:backupFranchiseFile (keeps 5 per file ID)", "Each open copies the save to a per-file backup folder, keeping the newest 5.", "", ST, "ui/files", "Six opens leave five timestamped copies with correct hashes.", "05c")
M("MF-FILE-10", "File session", "Open backup and data folders", S_MAIN + " fs:openBackupFolder, fs:openFranchiseDataFolder", "Opens the folder in the OS file manager.", "", TA, "ui/settings", "Opens on all three OSes via a scoped command.", "05c")
M("MF-FILE-11", "File session", "Delete a file's app data", S_R + " Delete File Data; fs:deleteFileData; franchise:deleteFileId", "Trash action removes local database and clears the embedded file ID after confirmation.", "MF-FILE-12", ST, "ui/files", "History is gone, progression can run again, and the save differs only in the ID field.", "05c")
M("MF-FILE-12", "File session", "File identity", S_MAIN + " franchise:getFileId writes FranchiseDebugModule.SideActivityToForce; SettingsPanel Copy ID", "A generated ID is stored in the save and keys all local data. Copy-to-clipboard is offered.", "", FO, "ui/settings", "ID survives rename; MaddenTool writes it only at first commit (deviation D-1); an existing MF ID is reused.", "05c")
M("MF-FILE-13", "File session", "Draft-pick table reorganization at load", S_MAIN + " franchise:reorganizeDraftPicks; load flow", "MF silently saves after consolidating draft picks into one table, then reports it.", "MF-FILE-15", FO, "ui/files", "MaddenTool previews this as a file-health fix and writes only on commit (deviation D-1).", "06a")
M("MF-FILE-14", "File session", "Repair missing or corrupt player visuals at load", S_MAIN + " franchise:repairVisuals; load flow", "MF silently saves repaired visuals, then reports it.", "MF-FILE-15", FO, "ui/files", "Previewed; a damaged fixture is repaired; untouched players are byte-identical in the visuals table.", "06a")
M("MF-FILE-15", "File session", "File-health report dialog", S_R + " File Saved dialog", "Lists what was changed on load.", "", FO, "ui/files", "Lists proposed fixes; nothing is written before consent.", "05c")
M("MF-FILE-16", "File session", "Custom player portrait", S_MAIN + " fs:setCustomPortrait, fs:removeCustomPortrait, fs:hasCustomPortrait; PlayerModal", "Pick an image for a player, shown app-wide; remove restores default. Local only; the save is not changed.", "", ST, "ui/players", "Set, restart, still shown; remove restores; save hash unchanged.", "06a")
M("MF-FILE-17", "File session", "Custom coach portrait", S_MAIN + " same channels with coach flag", "Same as players, in a coach folder.", "MF-FILE-16", ST, "ui/coaches", "Set and remove for a coach; save hash unchanged.", "06e")
M("MF-FILE-18", "File session", "Custom logo override folders", S_MAIN + " fs:createImageFolders, fs:getFranchiseLogoPathSync", "Global and per-file folders for teamNames, teamLogos, NCAA and teamLogos3D override images. Found in code only; no settings UI.", "", ST, "ui/shell", "Drop a logo in each folder level; per-file wins over global.", "08c")
M("MF-FILE-19", "File session", "Schema version resolution", S_MAIN + " oNt map 27:{965:620,987:640}", "The file's expected schema major selects a bundled schema through an override map.", "XP-014", FF, "ui/files", "965 and 987 open with the mapped schema; an unmapped major is refused for editing.", "05a")

# -------------------------------------------------------------------------- MF: shell
M("MF-SHELL-01", "Shell", "Grouped navigation", S_R + " VnS nav table", "Home, Teams, Schedule, Stats, Players, Coaches, Media, Draft, History and Tools with grouped menus.", "XP-005", "", "ui/shell", "Every nav item routes correctly; navigation collapses at 800x600.", "01b")
M("MF-SHELL-02", "Shell", "Back navigation stack", S_R + " NavBackButton", "Back returns through visited pages and resets on file change.", "", "", "ui/shell", "Sequence of navigations reverses exactly; disabled with no file loaded.", "05d")
M("MF-SHELL-03", "Shell", "Title bar context", S_R + " TeamNavbar; preload closeApp, minimizeApp", "Shows week label, version, team quick links, load/close, minimize and close.", "", TA, "ui/shell", "Week label matches SeasonInfo; native window controls work on all OSes.", "05d")
M("MF-SHELL-04", "Shell", "Contextual help", S_R + " FAQ data (47 blocks); FAQModal, FAQPage, FAQList, DefaultFAQ, GeneralFAQ, HomeFAQ", "Each page has a summary and Q&A. Original wording is required, and it must describe MaddenTool accurately.", "", "", "ui/help", "Each routed page opens help matching its real behaviour.", "08c")
M("MF-SHELL-05", "Shell", "Loading overlay, toasts and confirms", S_R + " LoadingOverlay, ModalBase, App", "Long operations block the affected work with a message; errors and confirmations are dialogs.", "XP-009", "", "ui/shell", "Replaced by job progress that leaves other screens usable (deliberate improvement).", "01b")
M("MF-SHELL-06", "Shell", "Update check, download and install", S_MAIN + " updater:*; LoadingOverlay; UpdateModal", "Checks releases, shows notes and progress, restarts to install.", "", TA, "ui/about", "Test endpoint offers an update and a tampered artifact is rejected. Needs signing keys (WP11b).", "11b")
M("MF-SHELL-07", "Shell", "App version display", "preload getAppVersion; TeamNavbar", "Version is shown in the shell.", "", TA, "ui/about", "Matches the package version.", "01b")
AVAIL = [
    ("LeagueLeaders", "/stats/leagueLeaders", "NFLSeason and week index > 0 (from Regular Season Week 2)"),
    ("PowerRankings", "/league/powerRankings", "always allowed once a file is loaded"),
    ("PlayoffBracket", "/league/playoffBracket", "Week 9 onward, or offseason before Draft Week"),
    ("WeeklyPreview", "/league/weeklyPreview", "during NFLSeason"),
    ("WeeklyRecap", "/league/weeklyRecap", "NFLSeason from Week 2"),
    ("SeasonPreview", "/league/predictions", "PreSeason, or NFLSeason at week index <= 0"),
    ("OffseasonPreview", "/league/offseasonPreview", "during OffSeason"),
    ("OffseasonReport", "/league/offseasonReport", "PreSeason, or NFLSeason at week index <= 0"),
    ("AwardWinners", "/awards/winners", "Super Bowl week or OffSeason"),
    ("PlayerOfTheWeek", "/awards/playerOfTheWeek", "NFLSeason from Week 2"),
    ("AwardsRace", "/awards/race", "week index 8 to 21 of NFLSeason"),
    ("ProBowlRosters", "/awards/proBowlRosters", "week index > 17 or OffSeason"),
    ("ProBowlVoting", "/awards/voting", "week index 8 to 17"),
    ("FreeAgencyTracker", "/players/freeAgencyTracker", "OffSeason stage > 1"),
    ("DraftBigBoard", "/draft/bigBoard", "NFLSeason from Week 5, or offseason before Draft Recap"),
    ("DraftClass", "/players/draftPlayers", "NFLSeason, or offseason before Draft Recap"),
    ("DraftOrder", "/draft/draftorder", "NFLSeason, or offseason before Draft Recap"),
    ("MockDraft", "/draft/mockDraft", "NFLSeason from Week 5, or offseason before Draft Recap"),
]
for i, (k, r, rule) in enumerate(AVAIL, 1):
    M(f"MF-AVAIL-{i:02d}", "Availability", f"Stage gate: {k}", S_R + f" oa() case {k}", f"{r} is available only: {rule}; otherwise an explanatory page (RouteUnavailableView).", "MF-FILE-19", FO, "ui/shell", "Table test over stage and week fixtures matches the rule; the unavailable page names when it opens.", "05d")

# -------------------------------------------------------------------------- MF: settings
for i, (op, beh, t) in enumerate([
    ("Theme", "Light or dark.", "Persists and applies to every route at AA contrast."),
    ("Tooltips", "Turn hover tooltips on or off.", "Persists; essential information stays reachable when off."),
    ("Table icons", "Tables show team and dev trait as names or logos.", "Persists; both modes render and sort."),
    ("Window size", "MF scales a fixed window (0.5 to 1.0, default 0.85).", "Satisfied by a resizable window and a UI zoom setting (deviation D-5)."),
    ("Warn on autosave", "Enable the autosave confirmation.", "Governs MF-FILE-05."),
    ("Auto-regenerate user depth charts", "Re-sort user depth charts on sign, cut or edit.", "Governs depth regeneration in player transactions."),
], 1):
    M(f"MF-SET-{i:02d}", "Settings", op, S_R + " SettingsPanel; options store", beh, "", ST, "ui/settings", t, "01b")

# -------------------------------------------------------------------------- MF: home
M("MF-HOME-01", "Home", "News feed", S_MAIN + " franchise:getNewsArticles, getStories, getTweets; Home*NewsCard", "Hero, featured, game and more-story cards built from the save's articles, stories and tweets.", READ, FO, "ui/home", "Fixture articles appear with correct targets; clicking opens the box score, preview or player.", "08c")
M("MF-HOME-02", "Home", "Ticker and game strip", S_MAIN + " franchise:getTicker; HomeNewsGameStrip", "Scrolling ticker and this week's scores.", READ, FO, "ui/home", "Ticker matches the save; reduced-motion stops scrolling.", "08c")
M("MF-HOME-03", "Home", "Regular-season hub", "RegularSeasonHub + widgets", "Phase-specific widgets: previews, recaps, awards, Pro Bowl, stats, power rankings.", "MF-HOME-01", FO, "ui/home", "Week 5 fixture shows the right widgets, each with non-empty or honest empty state.", "08c")
M("MF-HOME-04", "Home", "Preseason hub", "Preseason*Widget", "Season preview, news, report cards, players to watch and top available.", "MF-HOME-01", FO, "ui/home", "Preseason fixture shows these widgets.", "08c")
M("MF-HOME-05", "Home", "Offseason hub", "OffseasonHub, Offseason*Widget, DraftWeek*, StaffWeek*", "Staff week, re-sign, free agency, combine and draft week widgets by offseason stage.", "MF-HOME-01", FO, "ui/home", "One fixture per offseason stage shows the matching set. Fixtures must be captured.", "08c")
M("MF-HOME-06", "Home", "This week's tools panel", "WeeklyInfo, OffseasonWeeklyInfo, WeeklyTools; a91 spotlight", "Lists tools recommended for the current week.", "MF-TOOL-00", FO, "ui/home", "Matches each tool's spotlight schedule at sampled weeks.", "09a")

# -------------------------------------------------------------------------- MF: teams
M("MF-TEAM-01", "Teams", "Set team control", "/teams; LeagueTeams, TeamCard; franchise:setUserControl; FAQ", "Set each team to CPU, USER, COMM or OWNER; one owner; a head coach is required.", "F-WRITE", FO, "ui/teams", "Each transition reopens correctly and invalid ones are refused. In-game check needed.", "06d")
M("MF-TEAM-02", "Teams", "32-team control and bulk presets", "TeamSettingModal Bulk Control; FAQ", "Set control over all eligible teams; a 32 user cap; ownership transfer.", "MF-TEAM-01", FO, "ui/teams", "Cap message at 32; free-agent coach case matches the FAQ; ownership transfer leaves exactly one owner.", "06d")
M("MF-TEAM-03", "Teams", "User-team settings", "TeamSettingModal Is*Enabled fields", "Full Control, Simple and Custom presets over 16 per-user toggles.", "MF-TEAM-01", FO, "ui/teams", "Each toggle changes only its field; presets lock individual controls.", "06d")
for i, (tab, comps, beh, wp) in enumerate([
    ("Overview", "TeamOverview", "In-season matchup, standings, leaders, staff, injuries and stats; offseason needs, draft, free agents, outlook.", "08b"),
    ("Schedule", "TeamSchedule", "Team schedule with results.", "08b"),
    ("Statistics", "TeamPageStats", "Team stat views.", "08a"),
    ("Roster", "TeamRoster", "Team roster opening player cards.", "05d"),
    ("Injuries", "TeamInjuries", "Team injury report.", "05d"),
    ("Staff", "TeamStaff", "Team coaches and hire or fire entry points.", "06e"),
    ("Finances", "TeamPageFinances", "Team cap by year.", "06b"),
    ("Record books", "TeamRecordBooks", "Team records.", "08e"),
], 4):
    M(f"MF-TEAM-{i:02d}", "Teams", f"Team page: {tab}", f"/team/:id; TeamPage tab; {comps}", beh, READ, FO, "ui/teams", f"{tab} tab matches golden capture for a fixture team, including an empty state.", wp)
M("MF-TEAM-12", "Teams", "Depth chart edit", "TeamDepthChart, DepthChart*; franchise:editTeamDepthChart", "Reorder per position with save, cancel and unsaved-change warning.", "F-WRITE", FO, "ui/teams", "Reorder by mouse and keyboard; reopen shows the order; in-game lineup check.", "06d")
M("MF-TEAM-13", "Teams", "Depth chart auto-reorder", "TeamDepthChart Auto Reorder", "Generates a lineup by Madden's logic for one team.", "MF-TEAM-12", FO, "ui/teams", "Matches golden result for fixture team; injured starters are handled.", "06d")
M("MF-TEAM-14", "Teams", "Retired numbers", "TeamRetiredNumbers; retiredNumbers store; db:ensureRetiredNumbersTable, getRecord, checkAndUpdateRecord", "Retire or unretire jersey numbers per team, stored locally and used by Jersey Fix.", "MF-TOOL-14", ST, "ui/teams", "Numbers persist; Jersey Number Fix honours them.", "08e")

# -------------------------------------------------------------------------- MF: stats, schedule
M("F-FMT-01", "Franchise core", "FBCHUNKS container read and write", S_MAIN + " via madden-franchise 4.3.6; fixture probes (report section 5)", "zlib stream at 0x52, compressed length at 0x4a, fixed file size, trailing bytes after the stream retained.", "XP-014", FF, "ui/files", "No-op returns original bytes; recompressed output inflates to identical bytes; larger-than-capacity output refuses to write.", "05a")
M("F-FMT-02", "Franchise core", "FrTk tables, schema and references", "madden-franchise dist/index.cjs; M27_620_0.gz, M27_640_0.gz", "Table records, string and compressed-text tables, references, empty-record lists, enums.", "F-FMT-01;MF-FILE-19", FF, "ui/files", "Differential read of every table and field against the JS oracle; one-field edit matches oracle unpacked bytes; relationships verified.", "05b")
M("F-READ", "Franchise core", "Read model and queries", S_MAIN + " franchise:getPlayers, getTeams, getSeasonInfo, getSalaryInfo, getRosterInfo, getCoaches", "Paged queries over players, teams, coaches and season context.", "F-FMT-02", FO, "ui/query", "Row and field counts equal the oracle for the fixture; queries page; UI thread stays responsive during load.", "05d")
M("F-WRITE", "Franchise core", "Change-set write model", S_MAIN + " franchise:save (34 renderer call sites save immediately)", "Each MF action saves at once. MaddenTool applies to a session and commits transactionally (deviation D-2).", "XP-011;F-READ", FO, "ui/files", "One-field edit changes only expected unpacked bytes and the length header; unknown bytes preserved.", "06a")
M("MF-SCHED-01", "Schedule", "League schedule by week", "/schedule; LeagueSchedule*; franchise:getSeasonGames; FAQ", "Preseason and regular season always, playoffs when available; bye teams listed.", READ, FO, "ui/schedule", "Week switch matches SeasonGame in fixture.", "08b")
M("MF-SCHED-02", "Schedule", "Edit one game's weather", "WeatherEditModal; franchise:updateGameWeather", "Change forecast, wind and roof for an unplayed game.", "F-WRITE", FO, "ui/schedule", "Reopen shows the values; played games are not editable; in-game check.", "09a")
M("MF-SCHED-03", "Schedule", "Standings", "/standings; LeagueStandings, ConferenceStandings, DivisionStandings*", "Division, conference and league views; Expanded adds SOS and SOV.", READ, FO, "ui/schedule", "Order and SOS/SOV equal golden capture.", "08b")
M("MF-BOX-01", "Game", "Box score", "/boxScore/:gameId; BoxScore*, TeamBoxScorePlayerStats", "Team and player stats for a played game, plus division and upcoming widgets.", "F-READ", FO, "ui/game", "Totals match game stats tables for a fixture game.", "08b")
M("MF-BOX-02", "Game", "Gamecast view", "Gamecast, GameLeaders, PlayerOfTheGame, GameScore", "Alternate game summary with leaders and player of the game.", "MF-BOX-01", FO, "ui/game", "Matches golden capture for a played game.", "08b")
M("MF-STAT-01", "Stats", "Player stats by game, season, career", "/stats/leagueStats; LeagueStats, StatsDataTable and *StatsDataTable; franchise:getGameStats, getSeasonStats, getCareerStats, getMaxHistoricalSeason; HistoricalSeasonStats.json", "Eight stat types; season years back as far as the file holds.", READ, FO, "ui/stats", "Each type and scope matches oracle; historical year selector range is correct.", "08a")
M("MF-STAT-02", "Stats", "Season leaders", "/stats/leagueLeaders; LeagueLeaders*", "Current-season leaders by stat; row opens player.", "MF-STAT-01", FO, "ui/stats", "Top-N for each stat equals oracle.", "08a")
M("MF-STAT-03", "Stats", "Team stats", "/stats/teamStats; TeamStats, Team*StatsDataTable; franchise:getTeam*Stats", "Game (current), season (5 years) and career (W/L/T plus postseason).", READ, FO, "ui/stats", "Each scope matches oracle.", "08a")
M("MF-STAT-04", "Stats", "Team ratings", "/ratings/team; LeagueTeamRatings*", "Position-group ratings and Star, Superstar, X-Factor counts per team.", READ, FO, "ui/stats", "Counts and ratings equal oracle.", "08a")
M("MF-STAT-05", "Stats", "Edit player stats", "PlayerEditStatsModal, PlayerStatsModal; franchise:updateStats", "Edit career and season totals with unsaved warning.", "F-WRITE", FO, "ui/stats", "Edited value reopens; other stat rows unchanged.", "08a")
M("MF-STAT-06", "Stats", "Player stat records", "playerStatRecords store; franchise:getPlayerStatRecords", "Career highs and record lookups in player card.", READ, FO, "ui/stats", "Career highs equal oracle.", "08a")
M("MF-STAT-07", "Stats", "Rookie tracker", "RookieWatchListPanel; franchise:getRookieStatTracker", "Tracks rookie performance.", READ, FO, "ui/stats", "Matches save's tracker rows.", "08a")

# -------------------------------------------------------------------------- MF: player lists and card
for i, (op, route, comp, beh) in enumerate([
    ("Active players", "/players/players", "LeaguePlayers", "All active roster players."),
    ("Free agents", "/players/freeAgents", "FreeAgents", "All free agents."),
    ("Practice squads", "/players/practiceSquads", "PracticeSquadPlayers", "All practice-squad players."),
    ("Injured players", "/players/injuries", "LeagueInjuries", "Injured or IR players."),
    ("Retired players", "/players/retiredPlayers", "RetiredPlayers", "Retired players still in the file."),
    ("All players", "/players/allPlayers", "AllPlayers", "Every player in the file."),
], 1):
    M(f"MF-PL-{i:02d}", "Player lists", op, f"{route}; {comp}, PlayersDataTable, CustomDataTable; FAQ", beh + " Search, filter, sort; a row opens the player card.", READ, FO, "ui/players", "Membership equals oracle; filters and sort work on virtualized data.", "05d")
M("MF-PL-07", "Player lists", "Player search and switcher", "SearchPlayer, SwitchPlayer", "Global search and moving between players in the card.", READ, FO, "ui/players", "Finds by partial name; next and previous keep tab context.", "05d")
for i, (tab, c, beh, wp) in enumerate([
    ("Overview", "PlayerModalOverview, PlayerModalHeader, PlayerRatingBar", "Bio, core attributes, combine, contract, legacy, awards.", "05d"),
    ("DNA and traits", "PlayerTraitModal", "Persona DNA, focus, mindset, traits.", "05d"),
    ("Abilities", "PlayerAbility*, AbilityIcon*", "Ability slots and tiers.", "05d"),
    ("Statistics", "PlayerStatsModal", "Career, season, game, highs.", "08a"),
    ("Contract", "PlayerContractModal", "Yearly salary, bonus, cap hit, dead cap, void years, release and June 1 impact.", "06b"),
    ("Progression history", "PlayerProgressionModal; db:progression:getPlayerHistory", "Per-season rating changes from local history.", "09c"),
], 1):
    M(f"MF-CARD-{i:02d}", "Player card", f"Tab: {tab}", f"PlayerModal; {c}", beh, READ, FO, "ui/players", f"{tab} tab equals oracle for a star, rookie, retired and free-agent fixture.", wp)
for i, (op, c, beh, t) in enumerate([
    ("Edit bio and identifiers", "PlayerEditModal Player Info", "Name, age, position, jersey, height, weight, college, handedness, portrait, asset, commentary, presentation, injury and draft info.", "Each field round-trips; invalid values are rejected at entry."),
    ("Edit ratings", "PlayerEditModal Ratings, RatingField; ovrweights.json", "All attributes with overall recalculation.", "Overall equals oracle per position and archetype."),
    ("Edit DNA", "PlayerEditDnaModal; personaDNA.json", "Focus, mindset, DNA, temp DNA with duration.", "Slots and duration round-trip."),
    ("Edit traits", "PlayerEditTraitModal; playerTraits.json", "Behavioural traits.", "Each trait round-trips."),
    ("Edit abilities", "PlayerAbilitiesEditor, PlayerAbilitiesSelector; abilities.json", "Assign abilities and tiers within position rules.", "Position and range rules enforced; in-game check."),
    ("Edit appearance", "CharacterVisualsEditor, CharacterVisualsSelector, GenericHeadSelector; characterVisuals.json", "Head, body, equipment, vanity slots, auto left/right, advanced text mode.", "Visuals JSON round-trips; head change updates portrait ID; in-game check."),
    ("Edit contract", "PlayerContractEditor; franchise:editContract", "Length, year, void years, no-trade, guarantee, per-year salary and bonus, incentives.", "All 26 PlayerContract fields covered; cap totals equal oracle; incentive references intact."),
], 1):
    M(f"MF-EDIT-{i:02d}", "Player edit", op, c, beh, "F-WRITE", FO, "ui/players", t, "06b" if op == "Edit contract" else "06a")
for i, (op, ev, beh, t) in enumerate([
    ("Sign free agent", "PlayerSignModal; franchise:signPlayer", "Sign to a team with a contract.", "Roster, FA table, contract, transaction entry and depth chart all update."),
    ("Sign to practice squad", "franchise:signPlayerToPracticeSquad, isPracticeSquadFull", "Places player on a squad; refused when full.", "Full squad refused; references valid."),
    ("Promote from practice squad", "PlayerModal Actions", "Practice-squad player moves to active roster.", "Moves tables correctly; contract created."),
    ("Move to practice squad", "PlayerModal Actions; franchise:cutPlayer flags", "Active player moves to the squad.", "Tables and depth chart consistent."),
    ("Cut player", "franchise:cutPlayer", "Release with cap impact shown.", "Dead cap equals preview and oracle."),
    ("Cut with June 1 designation", "PlayerModal June 1st Designation", "Offseason cut splitting dead cap.", "Differs from a normal cut exactly per impact table."),
    ("Retire player", "franchise:retirePlayer", "Retires an unattached player.", "Moves to retired table; removed from FA."),
    ("Cut and retire", "PlayerModal Result: Cut + Retire", "Rostered player is cut then retired.", "Both effects applied atomically."),
    ("Unretire player", "franchise:unretirePlayer", "Back into free agency; visuals restored if needed.", "Appears in FA, not in retired."),
    ("Trade from card", "PlayerModal Trade Player", "Opens Trade Center with the player added.", "Trade Center opens preloaded."),
    ("Add or remove trade block", "PlayerModal; tradeBlock store", "Toggles the player's block status.", "Status round-trips; block page updates."),
], 1):
    M(f"MF-TXN-{i:02d}", "Player transactions", op, ev, beh, "F-WRITE;MF-TXN-20", FO, "ui/players", t + " Preview matches commit; in-game check.", "06c")
M("MF-TXN-20", "Player transactions", "Transaction logging", "/players/transactions; Transactions; franchise:getTransactionHistory, addTraded*; FAQ (740 stored)", "App actions append to the game's transaction history; page lists all.", "F-WRITE", FO, "ui/players", "Each action adds exactly one correct entry; capacity wraps as the game does.", "06c")
M("MF-FIN-01", "Finances", "Player contracts list", "/players/contracts; PlayerContracts", "Contract details for active players.", READ, FO, "ui/finance", "Values equal oracle.", "06b")
M("MF-FIN-02", "Finances", "Upcoming free agents", "/players/upcomingFreeAgents; UpcomingFreeAgents, UpcomingFreeAgentsHub", "Players expiring next offseason.", READ, FO, "ui/finance", "Membership equals oracle.", "06b")
M("MF-FIN-03", "Finances", "Team salaries", "/league/leaguefinancesview, /finances/league; LeagueFinances*; FAQ", "All teams' cap, roster counts and average age, this year plus 3.", READ, FO, "ui/finance", "Both routes render one view; each year equals oracle.", "06b")

# -------------------------------------------------------------------------- MF: coaches
for i, (op, route, c) in enumerate([
    ("Active coaches", "/coaches/coaches", "LeagueCoaches"), ("Free agent coaches", "/coaches/freeAgentCoaches", "FreeAgentCoaches"),
    ("Retired coaches", "/coaches/retiredCoaches", "RetiredCoaches"), ("All coaches", "/coaches/allCoaches", "AllCoaches"),
], 1):
    M(f"MF-CO-{i:02d}", "Coaches", op, f"{route}; {c}, CoachesDataTable; franchise:getCoaches", "Filterable list; row opens coach card.", READ, FO, "ui/coaches", "Membership equals oracle.", "05d")
M("MF-CO-05", "Coaches", "Coach card", "CoachModal, CoachModalHeader, CoachModalOverview; franchise:getCoachAwards", "Bio, record, contract, playbooks, schemes, talents, career stats.", READ, FO, "ui/coaches", "Equals oracle for HC, coordinator and free agent.", "06e")
M("MF-CO-06", "Coaches", "Edit coach", "CoachEditModal; franchise:editCoach", "Bio, position rules, playbooks, schemes, tendencies, career stats, appearance.", "F-WRITE", FO, "ui/coaches", "Fields round-trip; hired coach position locked.", "06e")
M("MF-CO-07", "Coaches", "Edit talents", "CoachEditTalent*, CoachTalentSection, TalentTile, TalentCardHeader, TalentDetailsPanel; franchise:editCoachTalents", "Add and change talents and tiers by category.", "F-WRITE", FO, "ui/coaches", "Talent references intact on reopen; in-game check.", "06e")
for i, (op, ch, beh) in enumerate([
    ("Hire coach", "franchise:hireCoach", "Assign a free-agent coach to a vacant role."),
    ("Fire coach", "franchise:fireCoach", "Release a coach from a team."),
    ("Retire coach", "franchise:retireCoach", "Retire a coach."),
    ("Unretire coach", "franchise:unretireCoach", "Return a coach to free agency."),
], 8):
    M(f"MF-CO-{i:02d}", "Coaches", op, ch + "; CoachModal", beh + " Transaction logged.", "F-WRITE;MF-TXN-20", FO, "ui/coaches", "Team, coach and transaction tables are consistent on reopen; in-game check.", "06e")
M("MF-CO-12", "Coaches", "Coach search and switcher", "SearchCoach, SwitchCoach", "Find and step through coaches.", READ, FO, "ui/coaches", "Finds by partial name.", "05d")

# -------------------------------------------------------------------------- MF: draft
M("MF-DR-01", "Franchise draft", "Draft order view", "/draft/draftorder; DraftOrder*, DraftOrderHub; franchise:getAllDraftPicks; FAQ", "Order by round, current and 2 future years; hover highlights team picks.", READ, FO, "ui/draft", "Order equals oracle for each year.", "07a")
M("MF-DR-02", "Franchise draft", "Edit pick owner", "DraftPickModal; franchise:editDraftPick", "Assign a pick to any team and show its value.", "F-WRITE;MF-FILE-13", FO, "ui/draft", "Owner round-trips; original team kept.", "07a")
M("MF-DR-03", "Franchise draft", "Draft class with spoiler toggle", "/players/draftPlayers; DraftPlayers*; franchise:getDraftPlayers; FAQ", "Prospects with combine and pro day; ratings and dev hidden until Show Ratings.", READ, FO, "ui/draft", "Hidden values are absent from the UI payload until enabled, including via sort.", "07a")
M("MF-DR-04", "Franchise draft", "Edit draft prospect", "franchise:editDraftPlayer", "Edit a prospect's data.", "F-WRITE", FO, "ui/draft", "Round-trips; in-game check.", "07a")
M("MF-DR-05", "Franchise draft", "Big board", "/draft/bigBoard; DraftBigBoard, DraftProspect*, draftBigBoard store; FAQ", "Game's board order, news, class snapshot and position strength.", READ, FO, "ui/draft", "Order and snapshot equal oracle.", "07a")
M("MF-DR-06", "Franchise draft", "Mock drafts", "/draft/mockDraft; MockDraft, MockPickCard; franchise:getMockDraftMap; FAQ", "All 7 rounds for available mocks.", READ, FO, "ui/draft", "Picks equal oracle per mock.", "07a")
M("MF-DR-07", "Franchise draft", "Workout results", "DraftWorkoutResultsWidget; draftWorkouts store", "Combine and pro day results.", READ, FO, "ui/draft", "Values equal the save.", "07a")
M("MF-DR-08", "Franchise draft", "Draft history", "/draft/history; DraftHistory*; draftHistory store; FAQ", "History built from players still in the file.", READ, FO, "ui/draft", "Equals oracle; missing draft teams shown honestly.", "08e")

# -------------------------------------------------------------------------- MF: trades, FA
M("MF-TR-01", "Trades", "Trade block page", "/players/tradeBlock; TradeBlock route component; tradeBlock store", "Players currently on the block.", READ, FO, "ui/trades", "Membership equals oracle.", "07b")
M("MF-TR-02", "Trades", "Build a trade", "/league/tradeCenter; TradeCenter, TradeAssetSelectorPanel, TradeItemList, TeamTradeHeader, TradePlayersDataTable", "Pick two teams and any number of players and picks.", READ, FO, "ui/trades", "Assets filter per team; reset clears.", "07b")
M("MF-TR-03", "Trades", "Trade valuation", "trade store; static/player/trades/*.json; TradeInterestBar, TradeSummarySection", "Interest and value by position, age, years pro, overall, cap and pick.", "MF-TR-02", FO, "ui/trades", "Scores equal WP00R goldens on a sampled trade set; provenance and applicable terms recorded for incorporated table data.", "07b")
M("MF-TR-04", "Trades", "Execute trade", "TradeCenter Confirm; addTradedPlayerTransaction, addTradedDraftPickTransaction", "Moves players and picks, records transactions.", "F-WRITE;MF-TR-03;MF-TXN-20", FO, "ui/trades", "Multi-asset trade atomic on reopen; cap and roster consistent; in-game check.", "07b")
M("MF-TR-05", "Trades", "Override trade", "TradeCenter Override Trade", "Force a denied trade.", "MF-TR-04", FO, "ui/trades", "Denied without override, allowed with it.", "07b")
M("MF-FA-01", "Free agency", "Free agency tracker", "/players/freeAgencyTracker; FreeAgencyTracker; franchise:getFreeAgencyTrackerData; FAQ", "Offseason signings.", READ, FO, "ui/offseason", "Equals save data.", "07c")

# -------------------------------------------------------------------------- MF: media
for i, (op, c, beh) in enumerate([
    ("Season preview: predictions", "Predictions, PredictionsFullPanel, PredictionsPlayoffSeedsColumn, PredictionAwardWinnerRow, SeasonPreviewSuperBowlSlide, SeasonPreviewAwardWinnersSlide", "Run Simulation yields division winners, playoffs and awards."),
    ("Season preview: team outlooks", "SeasonPreviewTeamOutlooksPanel, TeamOutlook*, SeasonPreviewTeamUnitRankingsPanel, SeasonPreviewTopRatedTeamSlide", "Power order, top players, starters, draft class, moves."),
    ("Season preview: NFL Top 100", "SeasonPreviewTop100Panel, SeasonPreviewTop100Slide", "Top 100 from last-season stats, overall and position value."),
    ("Season preview: players to watch", "SeasonPreviewPlayersToWatchPanel", "MVP candidates, contract years, new teams; breakout only if progression ran."),
    ("Season preview: rookie watch", "SeasonPreviewRookieWatchPanel, SeasonPreviewTopRookiesSlide", "Top rookies, steals, QBs, day-1 starters, dev totals."),
    ("Season preview: game of the week", "SeasonPreviewGamesOfWeekPanel", "One game per week; primetime totals."),
], 1):
    M(f"MF-MEDIA-{i:02d}", "Media", op, "/league/predictions; seasonPreview store; " + c, beh, READ, FO, "ui/media", "Deterministic ordering equals goldens; simulation is seeded and repeatable with a recorded seed (MF random behaviour characterized, not byte-matched).", "08d")
M("MF-MEDIA-07", "Media", "Power rankings: teams", "/league/powerRankings; PowerRankings, PowerRankTeam, PowerRankingsTeamSnapshot", "Record plus overall; preseason uses last season.", READ, FO, "ui/media", "Order equals golden in preseason and season fixtures.", "08d")
M("MF-MEDIA-08", "Media", "Power rankings: QBs", "PowerRankingsQbSnapshot", "Attributes plus statistical grade.", READ, FO, "ui/media", "Order equals golden.", "08d")
M("MF-MEDIA-09", "Media", "Playoff bracket and picture", "/league/playoffBracket; PlayoffBracket*, PlayoffPicturePanel; playoffBracket store; FAQ", "Projected, current or final bracket with Super Bowl MVP and box score links.", READ, FO, "ui/media", "Golden for projected, in-playoffs and post-Super-Bowl fixtures.", "08d")
M("MF-MEDIA-10", "Media", "Weekly preview", "/league/weeklyPreview; WeeklyPreview*; weeklyPreview store; FAQ", "Matchup screen: QB and HC, lineups, leaders, injuries, ratings, weather, O/U, network, stadium.", READ, FO, "ui/media", "Golden for a fixture game; current week only.", "08d")
M("MF-MEDIA-11", "Media", "Weekly recap", "/league/weeklyRecap; WeeklyRecap, GameRecap, WeeklyTopPerformersWidget; weeklyRecap store; FAQ", "Scores, stories, POTW, injuries, leaders, top performances for past weeks.", READ, FO, "ui/media", "Golden for two different weeks.", "08d")
for i, (op, c, beh) in enumerate([
    ("Overview", "OffseasonTrackerOverviewPanel, OffseasonTrackerFeatureCard, OffseasonTrackerHeadlinerCard", "Top moves summary."),
    ("Team tracker", "OffseasonTrackerTeamOutlookPanel", "Each team's results, needs, moves, picks."),
    ("Retirement", "OffseasonTrackerRetirementPanel", "Retirees with career splits."),
    ("Coaching carousel", "OffseasonTrackerCoachingPanel, OffseasonTrackerCoachRoleCard, OffseasonTrackerRetainedCoachCard", "Coaching changes and top candidates."),
    ("Free agency", "OffseasonTrackerFreeAgentsPanel", "Top 50 and by position, with signing details."),
    ("Draft preview", "OffseasonTrackerDraftPanel, OffseasonTrackerDraftRecapPanel", "Top 10 board, #1 pick, round 1 order, draft capital."),
], 1):
    M(f"MF-OFF-{i:02d}", "Offseason", f"Offseason tracker: {op}", "/league/offseasonPreview; offseasonPreview store; FAQ; " + c, beh, READ, FO, "ui/offseason", "Golden at matching offseason stage fixture.", "07c")
M("MF-OFF-07", "Offseason", "Offseason grades", "/league/offseasonReport; OffseasonReport*; offseasonReport store; FAQ", "Grades for all 32 teams from moves, draft and needs.", READ, FO, "ui/offseason", "Grades equal goldens on a preseason fixture.", "07c")
for i, (op, route, c, beh) in enumerate([
    ("Player of the week", "/awards/playerOfTheWeek", "PlayerOfTheWeek, PlayerOfTheWeekWinnerTile", "Current-season POTW awards, read-only."),
    ("Awards race", "/awards/race", "AwardsRace, AwardsRaceTable, AwardRaceSpotlightWidget", "Top contenders from current stats."),
    ("Award winners", "/awards/winners", "AwardWinners, AwardWinnerCard, AwardWinnersWidget", "Current winners and stats, read-only."),
    ("Pro Bowl voting", "/awards/voting", "ProBowlVoting, ProBowlVotingTable, ProBowlVotingSpotlightWidget", "Votes per position from performance and overall."),
    ("Pro Bowl rosters", "/awards/proBowlRosters", "ProBowlRosters, ProBowlRosterCard, ProBowlRosterCondensedRow", "Rosters and team summary, read-only."),
], 1):
    M(f"MF-AWD-{i:02d}", "Awards", op, f"{route}; {c}; award store; franchise:getPlayerAwards", beh, READ, FO, "ui/awards", "Equals golden at a stage where it is available.", "08e")
M("MF-HIST-01", "History", "Progression report", "/tools/progression-report; ProgressionReport lazy chunk; db:progression:getReport, getSeasonYears; FAQ", "All historical progression runs; filter by year, type, direction, position.", "MF-TOOL-01", ST, "ui/history", "After a run, rows equal snapshot deltas; empty state before any run.", "09c")
M("MF-HIST-02", "History", "Super Bowl history", "/league/superBowlHistory; SuperBowlHistory*; leagueHistory store; franchise:getLeagueHistory; LeaguePastHistory.json", "All results and team counts; winner links to Season Recap.", READ, FO, "ui/history", "Past plus in-save rows equal oracle; provenance and applicable terms recorded for incorporated data.", "08e")
M("MF-HIST-03", "History", "Awards history", "/league/awardsHistory; AwardsHistory; LeaguePastAwards.json", "All award winners, read-only.", READ, FO, "ui/history", "Equals oracle.", "08e")
M("MF-HIST-04", "History", "League and team records", "/league/recordBooks; RecordBooks, RecordBookPanels", "NFL records with a team filter, read-only.", READ, FO, "ui/history", "Equals oracle.", "08e")
M("MF-HIST-05", "History", "Season recap", "/league/history; SeasonRecap", "Super Bowl winner and award winners per season.", READ, FO, "ui/history", "Equals oracle per season.", "08e")
M("MF-LHUB-01", "League hub", "League hub widgets", "LeagueHub, LeagueHubCapSpace, LeagueHubStatLeaders, LeagueHubRetiredPlayers, TeamStatsWidget", "Summary widgets on home surfaces.", READ, FO, "ui/home", "Golden on a fixture.", "08c")

# -------------------------------------------------------------------------- MF: tools
M("MF-TOOL-00", "Tools", "Tools hub", "/tools; Tools, ToolCard, ToolSchedule, ToolsOptionsModal, ToolResultsTable; mI registry; FAQ", "18 tools by category; search; filters; this-week spotlight; full schedule; per-tool options; allowed-stage guards.", READ, FO, "ui/tools", "Spotlight and stage guards match each tool's metadata at sampled weeks; a stage-blocked tool explains why.", "09a")
TOOLS = [
    ("Progression Tool", "ProgressionTool", "Pre-Season Week 1 to Week 18, once per season. Controlled-random progression toward generated potentials; replaces in-game XP (sliders at 0).", "09c"),
    ("Offseason Progression Tool", "OffseasonProgressionTool", "Offseason only, once. Ratings from last season's stats versus expectation.", "09c"),
    ("Injured Reserve Fix", "InjuredReserveFix, InjuryToolPreviewModal", "Removes uninjured players from IR and places long-term injuries on IR. 2.0.1 ignores the stated >=70 OVR limit; 2.0.2 adds it. Use the 2.0.2 filter (deviation D-3).", "09a"),
    ("Remove All Injuries", "RemoveAllInjuries, InjuryToolPreviewModal", "Clears all injuries, returns uninjured IR players; options for user teams and depth charts.", "09a"),
    ("Weather Generator", "WeatherGenerator; static/weather", "Realistic weather for all games; rerun weekly in playoffs.", "09a"),
    ("Shuffle Coach Wardrobe", "ShuffleCoachWardrobe; static/coach/visuals/shuffleWardrobe", "Randomizes active coach shirts.", "09a"),
    ("Update Trade Block", "TradeBlock tool component (lX1, name collision), tradeBlock store", "Adds candidates by contract, morale, tags, team context, protections.", "09b"),
    ("Recalculate Draft Order", "RecalculateDraftOrder; franchise:reorderDraftPicks", "NFL order with tiebreak snaking; fixes duplicates; offseason weeks 1-9.", "09b"),
    ("Presentation ID Fix", "PresentationIdFix", "Presentation IDs for selected or rookie players.", "09a"),
    ("Commentary ID Fix", "CommentaryIdFix; static/player/commentary.json", "Last-name commentary IDs by player group.", "09a"),
    ("Free Agent Lottery", "FreeAgencyLottery", "Shows interested teams, lets the user edit them, then randomly signs.", "09b"),
    ("Training Camp Roster Fill", "TrainingCamp", "Previews and signs eligible FAs to CPU rosters.", "09b"),
    ("Trade Lottery", "TradeLottery", "Finds interested teams, builds pick packages, runs a lottery with preview.", "09b"),
    ("Jersey Number Fix", "JerseyNumberFix", "Position rules and retired numbers.", "09a"),
    ("Clear Cap Space", "ClearCapSpace", "Recommends cuts and restructures with savings, penalties, reasons; user can exclude items.", "09b"),
    ("Balance Rosters", "BalanceRosters", "Previews opposite-side position moves.", "09b"),
    ("Match Playbooks", "MatchPlaybooks", "Matches HC playbooks and schemes to coordinators.", "09b"),
    ("Reorder Depth Charts", "ReorderDepthCharts", "All-team depth reorder with exclusion option.", "09a"),
]
for i, (t, c, beh, wp) in enumerate(TOOLS, 1):
    M(f"MF-TOOL-{i:02d}", "Tools", t, f"mI registry; {c}", beh, "MF-TOOL-00;F-WRITE", FO, "ui/tools", "Preview equals commit; seeded run repeatable; golden comparison on fixture; cancel leaves source intact; in-game check.", wp)

# -------------------------------------------------------------------------- draft generator
LOOK = "server/src/services"
D("DG-DATA-01", "Historical data", "Draft years 1936-2026", "GET /draft/years; PlayerLookupService; ALL_PLAYER_LOOKUP.csv; YearPicker", "Year list with class sizes; about 32k players in 91 drafts.", "", GEN, "ui/classes", "Year and player counts equal the oracle dataset.", "03a")
D("DG-DATA-02", "Historical data", "AFL and NFL merge", "PlayerLookupService; README", "1960-1969 merged; league filter.", "DG-DATA-01", GEN, "ui/classes", "1965 merged counts and order equal oracle.", "03a")
D("DG-DATA-03", "Historical data", "Data acquisition, cache, degraded state", "NflverseCareerService, NflverseStatsService; README first-run; GET /config; MetaStrip", "First launch downloads data; classes built before it finishes are marked degraded.", "XP-009", GEN, "ui/classes", "Start with empty cache and no network: honest state; with network: progress, then regeneration clears degraded.", "03a")
D("DG-DATA-04", "Historical data", "Offline use after acquisition", "cache layout; README", "Works offline once data exists.", "DG-DATA-03", GEN, "ui/classes", "Block network after acquisition; generation still works.", "03a")
D("DG-DATA-05", "Historical data", "Supplemental and special players", "SupplementalDraftService, HistoricalAccoladeService, TwoWayService, FrontSevenService, FrontSevenClassifier, CuratedDbPositions, DbPositionService, TeamDraftService, WikipediaTeamService, TeamService, ReferenceDraftService, AwardsService", "Supplemental picks, two-way players, position curation, draft teams, accolades.", "DG-DATA-01", GEN, "ui/classes", "Port existing test cases for each service against oracle output.", "03c")
for i, (op, svc, beh) in enumerate([
    ("Launch Day rating lens", "LaunchRatingsService, M27RookieRatingsService", "Launch-style ratings."),
    ("Realistic rating lens", "RatingService, CalibrationService", "Ratings as scouted (mode madden)."),
    ("Career rating lens", "RatingService, RecomputeService", "Ratings from career results (mode retro)."),
], 1):
    D(f"DG-RATE-0{i}", "Ratings", op, f"View > Rating lens; POST /draft/recompute, recompute-batch; {svc}", beh + " Switch recomputes without reload.", "DG-DATA-01", GEN, "ui/classes", "Representative years equal TS oracle within recorded tolerance; lens change keeps selection.", "03b")
D("DG-RATE-04", "Ratings", "Hindsight control", "DraftOptions slider; POST /draft/custom hindsight", "0 is draft-day order, 1 is full hindsight.", "DG-RATE-01", GEN, "ui/classes", "Endpoints and midpoint equal oracle.", "03b")
D("DG-RATE-05", "Ratings", "Class strength, studs, generational options", "DraftOptions; GenOptions strength, studs, generational, autoStrength, variant", "Scale class quality; force first pick X-Factor; variant reroll.", "DG-RATE-01", GEN, "ui/classes", "Seed and options reproduce oracle output.", "03b")
D("DG-RATE-06", "Ratings", "OVR, attribute, combine, archetype model", "OVRWeightsCalculator, AttributeModel, CombineService, ArchetypeService, PositionMapper, PositionLabel; M27 overrides", "Position and archetype-correct overall and attributes.", "DG-RATE-01", GEN, "ui/classes", "Recompute match rate per position at or above oracle (legacy test bar 97%).", "03b")
D("DG-RATE-07", "Ratings", "Development traits", "DevTraitService; m27-rookie-dev-traits.tsv", "Normal, Star, Superstar, X-Factor assignment.", "DG-RATE-06", GEN, "ui/classes", "Distribution equals oracle.", "03b")
D("DG-RATE-08", "Ratings", "Persona DNA generation", "PersonaService; m27-persona-freq.json; GET /lookups/persona-dna", "Five DNA slots; game-distribution sampling. Game files carry 5 in 1,555/1,556 older samples and 4 on one TEMP2 row.", "DG-RATE-06", GEN, "ui/classes", "Seeded sample equal to oracle; WinAtAllCosts exclusion retested in game.", "03b")
D("DG-RATE-09", "Ratings", "M27 hidden and identity fields", "M27Fields.ts; m27-field-stats.json", "Personality, focus, hidden bytes, birthdate, commentary and PID assignment.", "DG-RATE-06;DG-FILE-01", GEN, "ui/classes", "Values fall in observed game distributions on TEMP1-3 and template.", "03c")
D("DG-CLS-01", "Class sources", "Single-year class", "GET /draft/:year, /draft/:year/generated; DraftClassBuilder, DraftEnrichment; DraftOptions This year", "Full class for a year.", "DG-RATE-01", GEN, "ui/classes", "2003, 1987 and 1965 semantic diff against oracle has only recorded differences.", "03c")
D("DG-CLS-02", "Class sources", "Random draft", "RandomDraft; draft routes", "Shuffles to a year, with undo and put-back.", "DG-CLS-01", GEN, "ui/classes", "History and undo work.", "04c")
D("DG-CLS-03", "Class sources", "Decade class", "POST /draft/custom source=decade; DraftOptions", "Best of a decade.", "DG-CLS-01", GEN, "ui/classes", "Equals oracle.", "04c")
D("DG-CLS-04", "Class sources", "All-time class", "POST /draft/custom source=alltime", "Best of all years.", "DG-CLS-01", GEN, "ui/classes", "Equals oracle.", "04c")
D("DG-CLS-05", "Class sources", "By-team class", "POST /draft/custom source=team; GET /draft/franchises; TeamPicker", "Best players a franchise drafted, with relocations kept.", "DG-CLS-01", GEN, "ui/classes", "Equals oracle.", "04c")
D("DG-CLS-06", "Class sources", "Class Studio board", "ClassStudio, CatalogPanel, DraftBoard, GET /players/catalog, PoolCatalogService; POST /draft/custom source=picked", "Pool search, ordered board, saved named boards.", "DG-CLS-01", GEN, "ui/studio", "Board order persists; p95 search under 200 ms over the pool.", "04c")
D("DG-CLS-07", "Class sources", "Board reorder by mouse and keyboard", "ClassStudio; DraftBoard", "Reorder and remove picks.", "DG-CLS-06", "", "ui/studio", "Keyboard-only reorder matches drag result.", "04c")
D("DG-CLS-08", "Class sources", "Custom prospects", "ClassStudio drawer; validateCustomPlayer", "Create invented players with validation.", "DG-CLS-06", GEN, "ui/studio", "Invalid rejected; valid exports and reopens.", "04c")
D("DG-CLS-09", "Class sources", "Era-correct fillers", "GenericFillerService, EraRulesService, EraBioService, HometownService", "Short classes are padded from the era; toggle in studio.", "DG-CLS-06", GEN, "ui/studio", "Seeded fill equals oracle.", "03c")
D("DG-CLS-10", "Class sources", "Overflow visibility and recovery", "DroppedPanel, BoardStates; README 1987 had 554", "Players past the limit are listed and can be pulled in.", "DG-CLS-01", GEN, "ui/classes", "1987 shows the dropped list; pull-in swaps correctly and count stays at limit.", "04a")
D("DG-CLS-11", "Class sources", "Veteran and undrafted inclusion", "GenOptions include; ClassView; UndraftedBackfill tests; udfa_careers.json", "Include options add undrafted and veterans where present.", "DG-CLS-01", GEN, "ui/classes", "Toggling include changes membership per oracle.", "03c")
D("DG-CLS-12", "Class sources", "Class naming", "ClassName; POST /draft/custom name", "Named classes export under that name.", "DG-CLS-06", GEN, "ui/studio", "Name reaches CAREERDRAFT-<name>.", "04c")
D("DG-BRD-01", "Board", "Player table", "PlayerTable, ClassView, VirtualList, Toolbar, MetaStrip, PositionBreakdown", "Sort, filter and view a rated class; position breakdown.", "DG-CLS-01", GEN, "ui/classes", "Virtualized on 402 and 32k rows.", "04a")
D("DG-BRD-02", "Board", "Alternate board views", "BigBoard, WallBoard, PlayerCards", "Big board, wall and card layouts.", "DG-BRD-01", "", "ui/classes", "Filtering after scroll works, including one-result and empty results.", "04a")
D("DG-BRD-03", "Board", "Blind scouting and spoilers", "ClassView, BigBoard; README", "Opens blind; values read ? until Spoilers is on.", "DG-BRD-01", GEN, "ui/classes", "Hidden values are absent from UI payload; no sort, filter or tooltip path reveals them.", "04a")
D("DG-BRD-04", "Board", "Scouting notes", "ScoutingNotesService; m27-scouting-notes.json; scouting-phrases.json", "Generated scout read per prospect.", "DG-BRD-03", GEN, "ui/classes", "Seeded notes equal oracle and honour spoiler state.", "04a")
D("DG-EDT-01", "Player editing", "Profile card", "ProfileModal, RadarChart, GET /players/search, PlayerSearch", "Full player view with radar and scouting.", "DG-BRD-01", GEN, "ui/profile", "All tabs usable at 800x600 and by keyboard.", "04b")
D("DG-EDT-02", "Player editing", "Edit identity, bio, physicals, ratings, position, dev, persona", "PlayerEditPanel, ProfileModal; applyEdits; FocusEdit test", "Edits persist across navigation and apply to export.", "DG-EDT-01", GEN, "ui/profile", "Each field survives export and reopen.", "04b")
D("DG-EDT-03", "Player editing", "Undo and redo", "MenuBar Ctrl+Z, Ctrl+Shift+Z; web/src/cache.ts", "Full edit history.", "DG-EDT-02", "", "ui/profile", "Sequence reverses exactly; persisted and unsaved states differ visibly.", "04b")
D("DG-EDT-04", "Player editing", "Export, import, clear edits JSON", "MenuBar Edit menu", "Move edits between sessions; clear for a class.", "DG-EDT-02", ST, "ui/profile", "Round trip reproduces the same class bytes.", "04b")
D("DG-APP-01", "Appearance", "Face and head editor", "AppearanceEditor; GET /lookups/generic-heads, face-scans; FaceFeatures, PortraitService", "Generic heads and real scans with portrait ID.", "DG-EDT-01", GEN, "ui/appearance", "Head choice writes PID and asset per field map; in-game check.", "04d")
D("DG-APP-02", "Appearance", "Gear and equipment editor", "GearEditor; /gear/players*, GET /lookups/equipment, /gear-image; GearOptionsService, GearImageService, EraGearService, CuratedGearService, RealPlayerGearService", "Slot editing, era gear, copy from real player.", "DG-APP-01", GEN, "ui/appearance", "Only game-valid M27 assets written; loadout JSON round-trips.", "04d")
D("DG-APP-03", "Appearance", "Gear from photo", "POST /gear/from-photo; PhotoLookService", "Estimate gear from an image.", "DG-APP-02", GEN, "ui/appearance", "Sample images equal oracle within tolerance; bad image gives a clear error.", "04d")
D("DG-APP-04", "Appearance", "Likeness overrides", "/likeness/overrides GET, PUT, DELETE; LikenessOverrideService, LikenessService", "Persist per-player likeness choices.", "DG-APP-01", ST, "ui/appearance", "Survive restart and apply to export.", "04d")
D("DG-APP-05", "Appearance", "Skin tone resolution and tone from photo", "POST /likeness/tone-from-photo; SkinToneService, SkinToneClassify, CuratedSkinToneService, DerivedSkinToneService, WikiSkinToneService", "Curated, derived and photo-based tone.", "DG-APP-01", GEN, "ui/appearance", "Port tests equal oracle within tolerance.", "04d")
D("DG-APP-06", "Appearance", "Portrait and icon display", "/portrait/*, /image, /logo, /gear-image; RetroHeadshotService, RetroItaService, LogoService, PortraitFetchService", "Headshots, retro art, icons with fallback. Art is user-supplied or provenance-cleared.", "XP-015", IO, "ui/appearance", "Missing art shows fallback and never a broken image; app works with no bundled proprietary art.", "04d")
D("DG-FILE-01", "Class files", "M27 draft codec", "M27Parser.js, M27Writer.js, Mdc27Service; CAREERDRAFT-TEMP1-3 and template probed", "455 physical blocks, active count at 0x42, 5876-byte records. Patch-only writer replaces legacy rebuild.", "XP-011", DF, "ui/files", "No-op is byte-identical on TEMP1-3 and template (legacy fails, 389/389 visual blocks change); single edit changes only its bytes; malformed and truncated files refused.", "02a", status="behavior-verified")
D("DG-FILE-02", "Class files", "Open existing class", "OpenClass; POST /open/file, GET /open/:id, OpenedClassService", "Opens a file, shows it as a board, retains original bytes.", "DG-FILE-01", DF, "ui/files", "TEMP1-3 open as 389; a 402 class opens as 402.", "02c")
D("DG-FILE-03", "Class files", "Browse Saves for classes", "GET, POST /open/saves", "List CAREERDRAFT files in Saves.", "XP-004", DF, "ui/files", "Works when Saves is found; picker fallback otherwise.", "02c")
D("DG-FILE-04", "Class files", "Export class as new file", "POST /export/mdc download", "Writes a CAREERDRAFT file to a chosen path.", "DG-FILE-01", DF, "ui/files", "Reopen validates; imports in game.", "02c")
D("DG-FILE-05", "Class files", "Save to or back into Saves with .bak", "ExportMenu; export.ts tmp, .bak, rename", "Writes to Saves, keeping a backup.", "DG-FILE-04", DF, "ui/files", "Backup restorable; fault injection leaves prior file intact.", "02c")
D("DG-FILE-06", "Class files", "CSV export", "ExportMenu; web/src/csv.ts", "All attributes with BOM.", "DG-BRD-01", DF, "ui/files", "Column set and values equal oracle.", "04b")
D("DG-FILE-07", "Class files", "Active count limits", "writer header count; M27-PORT; game files", "Game writes 389. Tool writes 402. Physical 455.", "DG-FILE-01", DF, "ui/files", "Export above 402 blocked until an in-game import proves it; 402 import rechecked on RL4.", "02a", status="behavior-verified")
D("DG-ROS-01", "Rosters", "M27 roster file codec", "RosterFileService, RosterContainer, Tdb2Engine, RosterPositionService; ROSTER-* files", "Third format: FBCHUNKS with TDB2 and alternate layout. Not covered by franchise codec.", "XP-011", RF, "ui/rosters", "No-op on copied ROSTER fixture is byte-identical; one edit changes only its bytes.", "04e")
D("DG-ROS-02", "Rosters", "Open and browse roster", "/roster/saves, /roster/open, /roster/:id; RosterPicker, RostersView, TeamPanel, RosterEditor", "Open a roster and view teams.", "DG-ROS-01", RF, "ui/rosters", "Counts equal oracle.", "04e")
D("DG-ROS-03", "Rosters", "Historical roster builder", "POST /roster/build; RosterBuildService; RosterBuilder", "Build a historic roster document and export.", "DG-ROS-01;DG-CLS-01", RF, "ui/rosters", "Output opens in oracle; in-game load check.", "04e")
D("DG-ROS-04", "Rosters", "Add player preview and assign team", "POST /roster/preview-add; RosterAddService", "Generated player preview, team assignment.", "DG-ROS-03", RF, "ui/rosters", "Preview equals final added row; PID and portrait stored.", "04e")
D("DG-PACK-01", "Portraits", "Own-ID and custom-ID allocation", "PortraitPackService.assign; CustomPortraitIdService; m27-shipped-portrait-pids.json", "Kept M26 ID when M27 dropped it; else 60000-65535 registry.", "DG-FILE-01", IO, "ui/portraits", "Deterministic; registry exportable; shipped list recheck for RL4.", "10a")
D("DG-PACK-02", "Portraits", "DDS writing", "writeDds: 512x512 DX10 R8G8B8A8_UNORM_SRGB 1 mip, lanczos3 and unsharp", "Sampled output verified: 148 byte header, DXGI 29.", "DG-PACK-01", IO, "ui/portraits", "Header bytes equal oracle; pixel PSNR threshold against sharp output; MMC import accepted.", "10a")
D("DG-PACK-03", "Portraits", "Class subset pack", "POST /export/mdc portraitPack; write()", "Per-class folder, manifest and missing lists.", "DG-PACK-02", IO, "ui/portraits", "Class IDs equal pack file names; missing list accurate.", "10a")
D("DG-PACK-04", "Portraits", "Full pack", "POST /export/portrait-pack/all; writeFull; legacy cache held 8,673 DDS", "Every portrait the tool holds, resumable, cancelable.", "DG-PACK-02", IO, "ui/portraits", "Cancel and resume; UI stays responsive; respects sourced-art limit.", "10a")
D("DG-PACK-05", "Portraits", "Custom sources and CDN opt-in", "overrideFor; customSource; portrait-sources; PortraitFetchService", "Dropped files, disc headshots, opt-in ESPN/NFL fetch with host allowlist.", "DG-PACK-01", IO, "ui/portraits", "Priority order equals oracle; network off unless opted in.", "10a")
D("DG-PACK-06", "Portraits", "Other-mod collision detection", "otherModIds regex assets/added/plpo_id_; plugin string is plpo_ID_", "Skips IDs another .fbmod supplies. Unverified: no local .fbmod; regex case differs from plugin string; MMC reports tamper protection.", "DG-PACK-01", IO, "ui/portraits", "Test real .fbmod files from MMC 1.1.0.7 before enabling; otherwise a manual ID exclusion list.", "10b")
D("DG-PACK-07", "Portraits", "Legacy 2D recycled-slot pack", "POST /export/portraits/:year; PortraitModService, PortraitSlotService; File menu", "PNG folder named by recycled generic slot. M26-era Frosty flow.", "DG-PACK-02", IO, "ui/portraits", "Keep only if verified working on M27 in MMC; otherwise record the removal for user review (see report).", "10b", app="unverified on M27")
D("DG-FRAN-01", "Legacy franchise tools", "Cap reset", "POST /franchise/cap-reset; CapResetTool; flag gated", "Resets cap, new-file output.", "F-WRITE", FO, "ui/tools", "Re-derive against M27 PlayerContract fields; integrity and in-game check.", "09d", runtime=R_DG_F)
D("DG-FRAN-02", "Legacy franchise tools", "Bulk heal and dev set", "POST /franchise/player-edit; PlayerBulkTools", "Clear injuries; set dev tier by scope.", "F-WRITE", FO, "ui/tools", "Counts in preview equal commit.", "09d", runtime=R_DG_F)
D("DG-FRAN-03", "Legacy franchise tools", "Roster apply editor", "POST /franchise/players, roster-apply; RosterEditor", "Bulk edit franchise players.", "F-WRITE", FO, "ui/tools", "Only edited fields change.", "09d", runtime=R_DG_F)
D("DG-FRAN-04", "Legacy franchise tools", "Trait realism", "POST /franchise/trait-realism; DevTraitsTool", "Reassign dev traits realistically.", "F-WRITE", FO, "ui/tools", "Distribution equals oracle.", "09d", runtime=R_DG_F)
D("DG-FRAN-05", "Legacy franchise tools", "Free agent trim", "POST /franchise/trim-free-agents; FaTrimTool", "Trims FA pool.", "F-WRITE", FO, "ui/tools", "Removes only targets; relationships intact.", "09d", runtime=R_DG_F)
D("DG-FRAN-06", "Legacy franchise tools", "Draft pick reset", "POST /franchise/reset-draft-picks; DraftPickResetTool", "Restores picks.", "F-WRITE", FO, "ui/tools", "Ownership equals expected.", "09d", runtime=R_DG_F)
D("DG-FRAN-07", "Legacy franchise tools", "Schedule viewer", "POST /franchise/schedule; ScheduleViewer", "Read-only schedule.", "F-READ", FO, "ui/tools", "Merges with MF-SCHED-01 if results equal.", "09d", runtime=R_DG_F)
D("DG-FRAN-08", "Legacy franchise tools", "Relocate and rebrand", "POST /franchise/teams, relocate-rebrand; RelocationTool", "Change team identity and location.", "F-WRITE", FO, "ui/tools", "Team fields change only; in-game check.", "09d", runtime=R_DG_F)
D("DG-FRAN-09", "Legacy franchise tools", "Advance roster: age, retire, regress", "POST /franchise/advance-roster; AdvanceRosterTool", "Ages and retires by rules.", "F-WRITE", FO, "ui/tools", "Seeded result equals oracle.", "09d", runtime=R_DG_F)
D("DG-FRAN-10", "Legacy franchise tools", "Historic season preview and playoff arm", "/franchise/historic/seasons, preview, arm-playoffs; HistoricFranchiseService, SeasonPackService, HistoricSeasonTool", "Preview a historic season and set its era playoff format.", "F-WRITE", FO, "ui/tools", "Dry run equals commit; in-game playoff format check.", "09d", runtime=R_DG_F)
D("DG-FRAN-11", "Legacy franchise tools", "League history seeding", "LeagueHistoryService, LeagueHistoryData; scripts only; dirty probes", "No route or UI: developer script, still being probed in the dirty tree.", "F-WRITE", FO, "ui/tools", "Not parity-blocking unless user wants it: ship as tool after array format proof.", "09d", app="script-only", runtime=R_DG_F)
D("DG-SHELL-01", "App shell", "Home, menu, rail, top bar", "HomePage, MenuBar, SideRail, TopBar", "Area switching and File, Edit, View, Help menus.", "XP-005", "", "ui/shell", "Merged in the MaddenTool shell; every legacy command reachable.", "01b")
D("DG-SHELL-02", "App shell", "What's new and external links", "WhatsNew, CHANGELOG, MenuBar Help", "Release notes, report issue link.", "", "", "ui/about", "Notes from packaged data.", "11b")
D("DG-SHELL-03", "App shell", "Update banner", "UpdateBanner; desktop/main.cjs", "Background update, restart to install.", "MF-SHELL-06", TA, "ui/about", "Same updater as MF-SHELL-06.", "11b")
D("DG-SHELL-04", "App shell", "Error boundary and about", "ErrorBoundary; GET /about", "Crash containment and credits.", "XP-015", TA, "ui/about", "Thrown render error keeps app usable; credits list notices.", "01b")
D("DG-SHELL-05", "App shell", "Prewarm and client cache", "prewarm.ts, cache.ts", "Fast return to recent years; board persistence.", "XP-008", ST, "ui/query", "Replaced by Rust SQLite and query cache; year re-entry measured.", "03c")

# ---------------------------------------------------------------------- symbol maps
# MF components -> feature
C = {
 "MF-SHELL-01": "NavLink NavDropdown NavMenu",
 "MF-SHELL-02": "NavBackButton",
 "MF-SHELL-03": "TeamNavbar",
 "MF-SHELL-04": "DefaultFAQ FAQList FAQModal FAQPage GeneralFAQ HomeFAQ",
 "MF-SHELL-05": "LoadingOverlay",
 "MF-SHELL-06": "UpdateModal",
 "MF-AVAIL-01": "RouteUnavailableView",
 "MF-FILE-01": "HomeView",
 "MF-SET-01": "SettingsPanel SettingRow",
 "MF-HOME-01": "HomeFeaturedNewsCard HomeGameNewsCard HomeHeroNewsCard HomeMoreStoryNewsCard HomeMoreStoryTweetCard HomeNewsTextBlock HomeNewsTextContent HomeTopNewsCards",
 "MF-HOME-02": "HomeNewsGameStrip",
 "MF-HOME-03": "RegularSeasonHub GamePreviewWidget GameRecapsWidget HomeLeaderSpotlightCard HomeSuperBowlPreviewCard PowerRankingsWidget",
 "MF-HOME-04": "PreseasonNewsTabWidget PreseasonPlayersToWatchWidget PreseasonReportCardWidget PreseasonSeasonPreviewWidget PreseasonTopAvailableWidget",
 "MF-HOME-05": "OffseasonHub OffseasonCoachMovesWidget OffseasonFreeAgentCandidatesWidget OffseasonGradeReportCardWidget OffseasonHeadCoachCandidatesWidget OffseasonReSignedPlayersWidget OffseasonTradeBlockCandidatesWidget StaffWeekSuperBowlRecapWidget DraftReportCardWidget DraftWeekBoardWidget DraftWeekOrderPanel FreeAgencyReportCardWidget FreeAgencySigningsWidget KeyOffseasonMovesWidget OffseasonTrackerCombineWidget OffseasonTrackerDraftRecapWidget OffseasonTrackerFreeAgencyWidget OffseasonTrackerMockDraftWidget OffseasonTrackerPreviewWidget OffseasonTrackerProDayWidget OffseasonTrackerWidgetHeader SpotlightSurface",
 "MF-HOME-06": "WeeklyInfo OffseasonWeeklyInfo WeeklyTools",
 "MF-TEAM-01": "LeagueTeams TeamCard",
 "MF-TEAM-03": "TeamSettingModal",
 "MF-TEAM-04": "TeamOverview TeamOverviewInjuryReport TeamOverviewLeadersCard TeamOverviewMatchupCenter TeamOverviewOffseasonDraft TeamOverviewOffseasonFreeAgents TeamOverviewOffseasonNeeds TeamOverviewOffseasonOutlook TeamOverviewSchedule TeamOverviewStaffCard TeamOverviewStandings TeamOverviewStatsCard TeamPage TeamPageHeader",
 "MF-TEAM-05": "TeamSchedule",
 "MF-TEAM-06": "TeamPageStats",
 "MF-TEAM-07": "TeamRoster",
 "MF-TEAM-08": "TeamInjuries",
 "MF-TEAM-09": "TeamStaff",
 "MF-TEAM-10": "TeamPageFinances",
 "MF-TEAM-11": "TeamRecordBooks",
 "MF-TEAM-12": "TeamDepthChart DepthChartDropdown DepthChartPlayerCard",
 "MF-TEAM-14": "TeamRetiredNumbers",
 "MF-SCHED-01": "LeagueSchedule LeagueScheduleGame LeagueScheduleGameWeather",
 "MF-SCHED-02": "WeatherEditModal",
 "MF-SCHED-03": "LeagueStandings ConferenceStandings DivisionStandingTable DivisionStandings StandingsPanel StandingsHighlightsPanel",
 "MF-BOX-01": "BoxScore BoxScoreDivisionStandingsWidget BoxScoreHeader BoxScorePlayerStats BoxScorePlayerStatsTable BoxScoreTeamStats BoxScoreUpcomingGamesWidget TeamBoxScorePlayerStats TeamStatCompareBars ScoreHub",
 "MF-BOX-02": "Gamecast GameLeaders PlayerOfTheGame GameScore",
 "MF-STAT-01": "LeagueStats StatsDataTable BlockingStatsDataTable DefenseStatsDataTable KickStatsDataTable PassStatsDataTable PuntStatsDataTable ReceiveStatsDataTable ReturnStatsDataTable RushStatsDataTable SackCell StatTypeSelect",
 "MF-STAT-02": "LeagueLeaders LeagueLeadersTable",
 "MF-STAT-03": "TeamStats TeamStatsDataTable TeamConversionStatsDataTable TeamDefenseStatsDataTable TeamGeneralStatsDataTable TeamKickingStatsDataTable TeamOffenseStatsDataTable TeamPassStatsDataTable TeamPuntingStatsDataTable TeamReceiveStatsDataTable TeamReturnStatsDataTable TeamRushStatsDataTable TeamSpecialTeamStatsDataTable TeamStatsWidget",
 "MF-STAT-04": "LeagueTeamRatings LeagueTeamRatingsTable",
 "MF-STAT-05": "PlayerEditStatsModal",
 "MF-STAT-07": "RookieWatchListPanel",
 "MF-PL-01": "LeaguePlayers PlayersDataTable",
 "MF-PL-02": "FreeAgents",
 "MF-PL-03": "PracticeSquadPlayers",
 "MF-PL-04": "LeagueInjuries",
 "MF-PL-05": "RetiredPlayers RetiredPlayersSpotlightWidget",
 "MF-PL-06": "AllPlayers",
 "MF-PL-07": "SearchPlayer SwitchPlayer",
 "MF-CARD-01": "PlayerModal PlayerModalHeader PlayerModalOverview PlayerRatingBar PlayerPresentationImage",
 "MF-CARD-02": "PlayerTraitModal",
 "MF-CARD-03": "PlayerAbilityCard PlayerAbilityModal PlayerAbilitySlot AbilityFrame AbilityIcon AbilityIconWithFrame",
 "MF-CARD-04": "PlayerStatsModal",
 "MF-CARD-05": "PlayerContractModal",
 "MF-CARD-06": "PlayerProgressionModal",
 "MF-EDIT-01": "PlayerEditModal EditSection EditToggleField",
 "MF-EDIT-02": "RatingField",
 "MF-EDIT-03": "PlayerEditDnaModal",
 "MF-EDIT-04": "PlayerEditTraitModal",
 "MF-EDIT-05": "PlayerAbilitiesEditor PlayerAbilitiesSelector",
 "MF-EDIT-06": "CharacterVisualsEditor CharacterVisualsSelector GenericHeadSelector",
 "MF-EDIT-07": "PlayerContractEditor",
 "MF-TXN-01": "PlayerSignModal",
 "MF-TXN-20": "Transactions",
 "MF-FIN-01": "PlayerContracts",
 "MF-FIN-02": "UpcomingFreeAgents UpcomingFreeAgentsHub",
 "MF-FIN-03": "LeagueFinancesTable LeagueFinancesView",
 "MF-CO-01": "LeagueCoaches CoachesDataTable",
 "MF-CO-02": "FreeAgentCoaches",
 "MF-CO-03": "RetiredCoaches",
 "MF-CO-04": "AllCoaches",
 "MF-CO-05": "CoachModal CoachModalHeader CoachModalOverview",
 "MF-CO-06": "CoachEditModal CoachPositionSelect",
 "MF-CO-07": "CoachEditTalentCard CoachEditTalentModal CoachTalentSection TalentCardHeader TalentDetailsPanel TalentTile",
 "MF-CO-12": "SearchCoach SwitchCoach",
 "MF-DR-01": "DraftOrder DraftOrderHub DraftOrderPickCard DraftOrderRound DraftPicksDataTable",
 "MF-DR-02": "DraftPickModal",
 "MF-DR-03": "DraftPlayers DraftPlayersDataTable",
 "MF-DR-05": "DraftBigBoard DraftProspectCard DraftProspectHub",
 "MF-DR-06": "MockDraft MockPickCard",
 "MF-DR-07": "DraftWorkoutResultsWidget",
 "MF-DR-08": "DraftHistory DraftHistoryPickCard DraftHistoryRoundPickCard",
 "MF-TR-01": "TradeBlock TradeBlockSpotlightWidget",
 "MF-TR-02": "TradeCenter TradeAssetSelectorPanel TradeItemList TeamTradeHeader TradePlayersDataTable",
 "MF-TR-03": "TradeInterestBar TradeSummarySection",
 "MF-FA-01": "FreeAgencyTracker",
 "MF-MEDIA-01": "Predictions PredictionsFullPanel PredictionsPlayoffSeedsColumn PredictionAwardWinnerRow PredictionsSlideNav SeasonPreviewSuperBowlSlide SeasonPreviewAwardWinnersSlide",
 "MF-MEDIA-02": "SeasonPreviewTeamOutlooksPanel SeasonPreviewTeamUnitRankingsPanel SeasonPreviewTopRatedTeamSlide TeamOutlookFeatureCard TeamOutlookHeader TeamOutlookHeaderTile",
 "MF-MEDIA-03": "SeasonPreviewTop100Panel SeasonPreviewTop100Slide",
 "MF-MEDIA-04": "SeasonPreviewPlayersToWatchPanel",
 "MF-MEDIA-05": "SeasonPreviewRookieWatchPanel SeasonPreviewTopRookiesSlide",
 "MF-MEDIA-06": "SeasonPreviewGamesOfWeekPanel",
 "MF-MEDIA-07": "PowerRankings PowerRankTeam PowerRankingsTeamSnapshot",
 "MF-MEDIA-08": "PowerRankingsQbSnapshot",
 "MF-MEDIA-09": "PlayoffBracket PlayoffBracketChampionCard PlayoffBracketGameCard PlayoffPicturePanel",
 "MF-MEDIA-10": "WeeklyPreview WeeklyPreviewMetricTable WeeklyPreviewTeamPanel",
 "MF-MEDIA-11": "WeeklyRecap GameRecap WeeklyTopPerformersWidget",
 "MF-OFF-01": "OffseasonTracker OffseasonTrackerOverviewPanel OffseasonTrackerFeatureCard OffseasonTrackerHeadlinerCard OffseasonTrackerSlideNav OffseasonTrackerListPanel OffseasonTrackerMovementLine OffseasonTrackerPlayerLine",
 "MF-OFF-02": "OffseasonTrackerTeamOutlookPanel",
 "MF-OFF-03": "OffseasonTrackerRetirementPanel",
 "MF-OFF-04": "OffseasonTrackerCoachingPanel OffseasonTrackerCoachRoleCard OffseasonTrackerRetainedCoachCard",
 "MF-OFF-05": "OffseasonTrackerFreeAgentsPanel",
 "MF-OFF-06": "OffseasonTrackerDraftPanel OffseasonTrackerDraftPickLine OffseasonTrackerDraftRecapPanel",
 "MF-OFF-07": "OffseasonReport OffseasonReportCardPlayerLine OffseasonReportImpactPlayerCard OffseasonReportListPanel",
 "MF-AWD-01": "PlayerOfTheWeek PlayerOfTheWeekWinnerTile",
 "MF-AWD-02": "AwardsRace AwardsRaceTable AwardRaceSpotlightWidget",
 "MF-AWD-03": "AwardWinners AwardWinnerCard AwardWinnersWidget",
 "MF-AWD-04": "ProBowlVoting ProBowlVotingTable ProBowlVotingSpotlightWidget",
 "MF-AWD-05": "ProBowlRosters ProBowlRosterCard ProBowlRosterCondensedRow ProBowlGameWidget",
 "MF-HIST-01": "ProgressionReport ProgressionDirectionSelect ProgressionSeasonYearSelect ProgressionTypeSelect",
 "MF-HIST-02": "SuperBowlHistory SuperBowlMatchupTeam SuperBowlTeam",
 "MF-HIST-03": "AwardsHistory",
 "MF-HIST-04": "RecordBookPanels RecordBooks",
 "MF-HIST-05": "SeasonRecap",
 "MF-LHUB-01": "LeagueHub LeagueHubCapSpace LeagueHubRetiredPlayers LeagueHubStatLeaders",
 "MF-TOOL-00": "Tools ToolCard ToolResultsTable ToolSchedule ToolsOptionsModal",
 "MF-TOOL-01": "ProgressionTool",
 "MF-TOOL-02": "OffseasonProgressionTool",
 "MF-TOOL-03": "InjuredReserveFix InjuryToolPreviewModal",
 "MF-TOOL-04": "RemoveAllInjuries",
 "MF-TOOL-05": "WeatherGenerator",
 "MF-TOOL-06": "ShuffleCoachWardrobe",
 "MF-TOOL-08": "RecalculateDraftOrder",
 "MF-TOOL-09": "PresentationIdFix",
 "MF-TOOL-10": "CommentaryIdFix",
 "MF-TOOL-11": "FreeAgencyLottery",
 "MF-TOOL-12": "TrainingCamp",
 "MF-TOOL-13": "TradeLottery",
 "MF-TOOL-14": "JerseyNumberFix",
 "MF-TOOL-15": "ClearCapSpace",
 "MF-TOOL-16": "BalanceRosters",
 "MF-TOOL-17": "MatchPlaybooks",
 "MF-TOOL-18": "ReorderDepthCharts",
}
SHARED = """App BooleanDropdown BroadcastNetworkLogo CustomDataTable DevTraitCell DropdownField FormField Icon ModalBase ModalNumberField ModalTextField PageHeader PositionSelect RatingCell SeasonPeriodSelect SeasonWeekSelect SeasonWeekTypeSelect SeasonYearSelect TabbedContainer TeamDropdown TeamSelect TierBadge YearsProSelect""".split()
comp_map = {}
for fid, names in C.items():
    assert fid in ids, fid
    for n in names.split():
        assert n not in comp_map, n
        comp_map[n] = fid


def sym(sid, source, version, kind, symbol, cls, maps, ev, rt, note=""):
    assert sid not in ids, sid
    ids.add(sid)
    rows.append(dict(
        id=sid, row_type="symbol", source=source, source_version=version, family="symbol map",
        screen_or_operation=symbol, evidence=ev, observable_behavior=note, m27_applicability="",
        dependencies="", rust_owner="", ui_owner="", acceptance_test="covered by mapped feature" if maps else "",
        runtime_evidence=rt, status="", work_package="", pr_link="", symbol_kind=kind, symbol=symbol,
        classification=cls, maps_to=maps))


errors = []
for n in MF["vue_components"]:
    reach = REACH.get(n, [])
    ev = "static reach: " + (";".join(reach[:4]) + (f" +{len(reach)-4}" if len(reach) > 4 else "") if reach else "mI tool registry or lazy chunk")
    note = ""
    if n == "TradeBlock":
        note = "Two components share this name (page F3t and tool lX1); inventory collapsed them. Mapped to MF-TR-01 and MF-TOOL-07."
    if n == "LeagueStandings":
        note = "Two definitions share this name; inventory collapsed them."
    if n in comp_map:
        maps = comp_map[n] + (";MF-TOOL-07" if n == "TradeBlock" else "")
        sym("SYM-MF-C-" + n, "MyFranchise", MF_V, "component", n, "feature", maps, ev, "static-only", note)
    elif n in SHARED:
        sym("SYM-MF-C-" + n, "MyFranchise", MF_V, "component", n, "shared", "", ev, "static-only", note or "Shared control or app root; no separate outcome.")
    else:
        errors.append("unmapped component " + n)

ROUTE = {
 "/": "MF-FILE-01;MF-HOME-01", "/awards/playerOfTheWeek": "MF-AWD-01", "/awards/proBowlRosters": "MF-AWD-05", "/awards/race": "MF-AWD-02",
 "/awards/voting": "MF-AWD-04", "/awards/winners": "MF-AWD-03", "/boxScore/:gameId": "MF-BOX-01;MF-BOX-02", "/coaches/allCoaches": "MF-CO-04",
 "/coaches/coaches": "MF-CO-01", "/coaches/freeAgentCoaches": "MF-CO-02", "/coaches/retiredCoaches": "MF-CO-03", "/draft/bigBoard": "MF-DR-05",
 "/draft/draftorder": "MF-DR-01", "/draft/history": "MF-DR-08", "/draft/mockDraft": "MF-DR-06", "/finances/league": "MF-FIN-03",
 "/league/awardsHistory": "MF-HIST-03", "/league/history": "MF-HIST-05", "/league/leaguefinancesview": "MF-FIN-03",
 "/league/offseasonPreview": "MF-OFF-01", "/league/offseasonReport": "MF-OFF-07", "/league/playoffBracket": "MF-MEDIA-09",
 "/league/powerRankings": "MF-MEDIA-07;MF-MEDIA-08", "/league/predictions": "MF-MEDIA-01", "/league/recordBooks": "MF-HIST-04",
 "/league/superBowlHistory": "MF-HIST-02", "/league/tradeCenter": "MF-TR-02", "/league/weeklyPreview": "MF-MEDIA-10", "/league/weeklyRecap": "MF-MEDIA-11",
 "/players/allPlayers": "MF-PL-06", "/players/contracts": "MF-FIN-01", "/players/draftPlayers": "MF-DR-03", "/players/freeAgencyTracker": "MF-FA-01",
 "/players/freeAgents": "MF-PL-02", "/players/injuries": "MF-PL-04", "/players/players": "MF-PL-01", "/players/practiceSquads": "MF-PL-03",
 "/players/retiredPlayers": "MF-PL-05", "/players/tradeBlock": "MF-TR-01", "/players/transactions": "MF-TXN-20", "/players/upcomingFreeAgents": "MF-FIN-02",
 "/ratings/team": "MF-STAT-04", "/schedule": "MF-SCHED-01", "/standings": "MF-SCHED-03", "/stats/leagueLeaders": "MF-STAT-02", "/stats/leagueStats": "MF-STAT-01",
 "/stats/teamStats": "MF-STAT-03", "/team/:id": "MF-TEAM-04", "/teams": "MF-TEAM-01", "/tools": "MF-TOOL-00", "/tools/progression-report": "MF-HIST-01",
}
for r in MF["route_paths"]:
    if r not in ROUTE:
        errors.append("unmapped route " + r)
        continue
    note = "Alias: same component as /league/leaguefinancesview; not in nav." if r == "/finances/league" else ""
    sym("SYM-MF-R-" + r, "MyFranchise", MF_V, "route", r, "feature", ROUTE[r], "route table XQt", "static-only", note)

IPC = {
 "db:alterColumns": ("dead", "", "Exposed in preload; no renderer call found."),
 "db:checkAndUpdateRecord": ("feature", "MF-TEAM-14", ""), "db:close": ("feature", "MF-FILE-08", ""),
 "db:ensureRetiredNumbersTable": ("feature", "MF-TEAM-14", ""), "db:getRecord": ("feature", "MF-TEAM-14", ""),
 "db:initialize": ("shared", "MF-FILE-12", "Per-file SQLite open."),
 "db:progression:ensureTable": ("shared", "MF-TOOL-01", ""), "db:progression:getPlayerHistory": ("feature", "MF-CARD-06", ""),
 "db:progression:getReport": ("feature", "MF-HIST-01", ""), "db:progression:getSeasonYears": ("feature", "MF-HIST-01", ""),
 "db:progression:getSnapshot": ("feature", "MF-TOOL-01", ""), "db:progression:hasOffseasonRun": ("feature", "MF-TOOL-02", "Run-once guard."),
 "db:progression:hasRun": ("feature", "MF-TOOL-01", "Run-once guard."), "db:progression:insertPlayers": ("feature", "MF-TOOL-01;MF-TOOL-02", ""),
 "db:progression:saveSnapshot": ("feature", "MF-TOOL-01", ""),
 "db:syncTablesFromTemplate": ("dead", "", "Exposed in preload; no renderer call found."),
 "db:tableExists": ("dead", "", "Exposed in preload; no renderer call found."), "db:tableHasColumn": ("dead", "", "Exposed in preload; no renderer call found."),
 "franchise:addTradedDraftPickTransaction": ("feature", "MF-TR-04", ""), "franchise:addTradedPlayerTransaction": ("feature", "MF-TR-04", ""),
 "franchise:close": ("feature", "MF-FILE-08", ""), "franchise:cutPlayer": ("feature", "MF-TXN-05;MF-TXN-06;MF-TXN-04;MF-TXN-08", ""),
 "franchise:cutPracticeSquadPlayer": ("unresolved", "MF-TXN-05", "Exposed in preload and implemented in main, but no renderer call found. WP00R must show how a practice-squad cut is performed."),
 "franchise:deleteFileId": ("feature", "MF-FILE-11", ""), "franchise:editCoach": ("feature", "MF-CO-06", ""), "franchise:editCoachTalents": ("feature", "MF-CO-07", ""),
 "franchise:editContract": ("feature", "MF-EDIT-07", ""), "franchise:editDraftPick": ("feature", "MF-DR-02", ""), "franchise:editDraftPlayer": ("feature", "MF-DR-04", ""),
 "franchise:editPlayer": ("feature", "MF-EDIT-01;MF-EDIT-02;MF-EDIT-03;MF-EDIT-04;MF-EDIT-05;MF-EDIT-06", ""), "franchise:editTeamDepthChart": ("feature", "MF-TEAM-12;MF-TEAM-13", ""),
 "franchise:fireCoach": ("feature", "MF-CO-09", ""), "franchise:getAllDraftPicks": ("feature", "MF-DR-01;MF-TOOL-08", ""),
 "franchise:getCareerStats": ("feature", "MF-STAT-01", ""), "franchise:getCoachAwards": ("feature", "MF-CO-05", ""), "franchise:getCoaches": ("feature", "MF-CO-01;F-READ", ""),
 "franchise:getDraftPlayers": ("feature", "MF-DR-03", ""), "franchise:getFileId": ("feature", "MF-FILE-12", ""), "franchise:getFreeAgencyTrackerData": ("feature", "MF-FA-01", ""),
 "franchise:getGameStats": ("feature", "MF-STAT-01;MF-BOX-01", ""), "franchise:getLeagueHistory": ("feature", "MF-HIST-02;MF-HIST-05", ""),
 "franchise:getMaxHistoricalSeason": ("feature", "MF-STAT-01", ""), "franchise:getMockDraftMap": ("feature", "MF-DR-06", ""), "franchise:getNewsArticles": ("feature", "MF-HOME-01", ""),
 "franchise:getPlayerAwards": ("feature", "MF-AWD-03;MF-CARD-01", ""), "franchise:getPlayerStatRecords": ("feature", "MF-STAT-06", ""), "franchise:getPlayers": ("shared", "F-READ", ""),
 "franchise:getRookieStatTracker": ("feature", "MF-STAT-07", ""), "franchise:getRosterInfo": ("shared", "F-READ", ""), "franchise:getSalaryInfo": ("shared", "F-READ;MF-FIN-03", ""),
 "franchise:getSeasonGames": ("feature", "MF-SCHED-01", ""), "franchise:getSeasonInfo": ("shared", "F-READ", ""), "franchise:getSeasonStats": ("feature", "MF-STAT-01", ""),
 "franchise:getStories": ("feature", "MF-HOME-01", ""), "franchise:getTeamCareerStats": ("feature", "MF-STAT-03", ""), "franchise:getTeamGameStats": ("feature", "MF-STAT-03", ""),
 "franchise:getTeamSeasonStats": ("feature", "MF-STAT-03", ""), "franchise:getTeams": ("shared", "F-READ", ""), "franchise:getTicker": ("feature", "MF-HOME-02", ""),
 "franchise:getTransactionHistory": ("feature", "MF-TXN-20", ""), "franchise:getTweets": ("feature", "MF-HOME-01", ""), "franchise:hireCoach": ("feature", "MF-CO-08", ""),
 "franchise:isPracticeSquadFull": ("feature", "MF-TXN-02", ""), "franchise:open": ("feature", "MF-FILE-03", ""), "franchise:reload": ("feature", "MF-FILE-07", ""),
 "franchise:reorderDraftPicks": ("feature", "MF-TOOL-08", ""), "franchise:reorganizeDraftPicks": ("feature", "MF-FILE-13", ""), "franchise:repairVisuals": ("feature", "MF-FILE-14", ""),
 "franchise:retireCoach": ("feature", "MF-CO-10", ""), "franchise:retirePlayer": ("feature", "MF-TXN-07;MF-TXN-08", ""), "franchise:save": ("shared", "F-WRITE", "34 renderer call sites."),
 "franchise:setUserControl": ("feature", "MF-TEAM-01;MF-TEAM-02;MF-TEAM-03", ""), "franchise:signPlayer": ("feature", "MF-TXN-01;MF-TXN-03", ""),
 "franchise:signPlayerToPracticeSquad": ("feature", "MF-TXN-02;MF-TXN-04", ""), "franchise:unretireCoach": ("feature", "MF-CO-11", ""), "franchise:unretirePlayer": ("feature", "MF-TXN-09", ""),
 "franchise:updateGameWeather": ("feature", "MF-SCHED-02;MF-TOOL-05", ""), "franchise:updateStat": ("dead", "", "Exposed in preload; only updateStats is called."), "franchise:updateStats": ("feature", "MF-STAT-05", ""),
 "fs:backupFranchiseFile": ("feature", "MF-FILE-09", ""), "fs:createImageFolders": ("feature", "MF-FILE-18", ""), "fs:deleteFileData": ("feature", "MF-FILE-11", ""),
 "fs:franchiseMetaResult": ("feature", "MF-FILE-01", "Event."), "fs:getAppDataPath": ("shared", "MF-FILE-12", ""), "fs:getFranchiseLogoPathSync": ("feature", "MF-FILE-18", ""),
 "fs:getFranchisePortraitPathSync": ("feature", "MF-FILE-16;MF-FILE-17", ""), "fs:hasCustomPortrait": ("feature", "MF-FILE-16;MF-FILE-17", ""), "fs:openBackupFolder": ("feature", "MF-FILE-10", ""),
 "fs:openFileDialog": ("feature", "MF-FILE-03", ""), "fs:openFranchiseDataFolder": ("feature", "MF-FILE-10", ""), "fs:readFile": ("dead", "", "Exposed in preload; no renderer call found; arbitrary read must not be carried over."),
 "fs:readFolder": ("dead", "", "Exposed in preload; no renderer call found."), "fs:readMaddenSavesFolderBasic": ("feature", "MF-FILE-01", ""), "fs:removeCustomPortrait": ("feature", "MF-FILE-16;MF-FILE-17", ""),
 "fs:setCustomPortrait": ("feature", "MF-FILE-16;MF-FILE-17", ""), "fs:streamFranchiseMeta": ("feature", "MF-FILE-01", ""), "fs:writeFile": ("dead", "", "Exposed in preload; no renderer call found; arbitrary write must not be carried over."),
 "updater:check": ("feature", "MF-SHELL-06", ""), "updater:download": ("feature", "MF-SHELL-06", ""), "updater:downloaded": ("feature", "MF-SHELL-06", "Event."), "updater:error": ("feature", "MF-SHELL-06", "Event."),
 "updater:getVersion": ("dead", "", "Exposed in preload; no renderer call found; shell uses app:get-version."), "updater:install": ("feature", "MF-SHELL-06", ""), "updater:progress": ("feature", "MF-SHELL-06", "Event."),
}
for ch in MF["ipc_channels"]:
    if ch not in IPC:
        errors.append("unmapped ipc " + ch)
        continue
    cls, maps, note = IPC[ch]
    sym("SYM-MF-I-" + ch, "MyFranchise", MF_V, "ipc", ch, cls, maps, "preload.js + main.js handler", "static-only", note)
# channels missing from the inventory, found by this audit
for ch, cls, maps, note in [
    ("app:get-version", "feature", "MF-SHELL-07", "Missing from inventory."),
    ("app-close", "feature", "MF-SHELL-03", "Missing from inventory."),
    ("app-minimize", "feature", "MF-SHELL-03", "Missing from inventory."),
    ("app-resize", "feature", "MF-SET-04", "Missing from inventory."),
    ("save-settings", "feature", "MF-SET-04", "Missing from inventory."),
    ("franchise-file-changed", "feature", "MF-FILE-06", "Missing from inventory. Event."),
    ("franchise:getTableRecords", "dead", "", "Missing from inventory. Main handler with no preload exposure."),
]:
    sym("SYM-MF-I-" + ch, "MyFranchise", MF_V, "ipc-extra", ch, cls, maps, "main.js/preload.js audit", "static-only", note)
for t, note in [("assignAbilities", "Empty entry in mI; never shown."), ("exportSeasonData", "Empty entry in mI; never shown."), ("motivationsGenerator", "Empty entry in mI; never shown.")]:
    sym("SYM-MF-T-" + t, "MyFranchise", MF_V, "tool-registry", t, "dead", "", "mI registry", "static-only", note)

DR = {
 "/draft/years": "DG-DATA-01", "/draft/:year/generated": "DG-CLS-01", "/draft/recompute": "DG-RATE-01;DG-RATE-02;DG-RATE-03", "/draft/recompute-batch": "DG-RATE-01;DG-RATE-02;DG-RATE-03",
 "/draft/franchises": "DG-CLS-05", "/draft/custom": "DG-CLS-03;DG-CLS-04;DG-CLS-05;DG-CLS-06;DG-CLS-08", "/draft/:year": "DG-CLS-01",
 "/export/mdc": "DG-FILE-04;DG-FILE-05;DG-PACK-03", "/export/portrait-pack/all": "DG-PACK-04", "/export/portraits/:year": "DG-PACK-07",
 "/franchise/historic/seasons": "DG-FRAN-10", "/franchise/historic/arm-playoffs": "DG-FRAN-10", "/franchise/historic/preview": "DG-FRAN-10",
 "/franchise/list": "DG-FRAN-03", "/franchise/cap-reset": "DG-FRAN-01", "/franchise/player-edit": "DG-FRAN-02", "/franchise/players": "DG-FRAN-03", "/franchise/roster-apply": "DG-FRAN-03",
 "/franchise/teams": "DG-FRAN-08", "/franchise/trait-realism": "DG-FRAN-04", "/franchise/trim-free-agents": "DG-FRAN-05", "/franchise/reset-draft-picks": "DG-FRAN-06",
 "/franchise/schedule": "DG-FRAN-07", "/franchise/relocate-rebrand": "DG-FRAN-08", "/franchise/advance-roster": "DG-FRAN-09",
 "/gear/players/info": "DG-APP-02", "/gear/players": "DG-APP-02", "/gear/players/:id": "DG-APP-02", "/gear/from-photo": "DG-APP-03",
 "/health": None, "/config": "DG-DATA-03", "/template/info": None, "/about": "DG-SHELL-04",
 "/likeness/overrides": "DG-APP-04", "/likeness/overrides/export": "DG-APP-04", "/likeness/tone-from-photo": "DG-APP-05",
 "/lookups": "DG-EDT-02", "/lookups/archetypes-by-position": "DG-EDT-02", "/lookups/equipment": "DG-APP-02", "/lookups/generic-heads": "DG-APP-01", "/lookups/face-scans": "DG-APP-01", "/lookups/persona-dna": "DG-RATE-08", "/lookups/:name": "DG-EDT-02",
 "/image": "DG-APP-06", "/logo": "DG-APP-06", "/open/saves": "DG-FILE-03", "/open/file": "DG-FILE-02", "/open/:id": "DG-FILE-02",
 "/players/search": "DG-EDT-01", "/players/catalog": "DG-CLS-06", "/gear-image/:value": "DG-APP-02", "/portrait/generic-head/:code": "DG-APP-06", "/portrait/retro/:first/:last": "DG-APP-06",
 "/portrait/dev-icon/:name": "DG-APP-06", "/portrait/dna-icon/:name": "DG-APP-06", "/portrait/pid/:pid": "DG-APP-06", "/portrait/plpo/:plpo": "DG-APP-06",
 "/roster/build": "DG-ROS-03", "/roster/preview-add": "DG-ROS-04", "/roster/saves": "DG-ROS-02", "/roster/open": "DG-ROS-02", "/roster/:id": "DG-ROS-02",
}
for r in DG["routes"]:
    key = r["method"] + " " + r["path"]
    if r["path"] not in DR:
        errors.append("unmapped draft route " + key)
        continue
    m = DR[r["path"]]
    note = ""
    if r["path"].startswith("/franchise"):
        note = "Mounted only with DRAFT_TOOL_FRANCHISE=1."
    if r["path"] in ("/gear/players/info", "/template/info", "/likeness/overrides/export"):
        note = "No textual UI caller found."
    if m is None:
        sym("SYM-DG-R-" + key, "Draft generator", DG_V, "route", key, "shared", "", f"{r['file']}:{r['line']}", "code read", note or "Local server plumbing; replaced by Tauri commands.")
    else:
        sym("SYM-DG-R-" + key, "Draft generator", DG_V, "route", key, "feature", m, f"{r['file']}:{r['line']}", "code read", note)

DC = {
 "AppearanceEditor": "DG-APP-01", "BigBoard": "DG-BRD-02", "BoardStates": "DG-CLS-10", "CatalogPanel": "DG-CLS-06", "ClassStudio": "DG-CLS-06", "ClassView": "DG-BRD-01", "DraftBoard": "DG-CLS-07",
 "DraftOptions": "DG-RATE-04;DG-RATE-05;DG-CLS-03;DG-CLS-04;DG-CLS-05", "DroppedPanel": "DG-CLS-10", "ErrorBoundary": "DG-SHELL-04", "ExportMenu": "DG-FILE-04;DG-FILE-05;DG-FILE-06;DG-PACK-03;DG-PACK-04",
 "AdvanceRosterTool": "DG-FRAN-09", "CapResetTool": "DG-FRAN-01", "DevTraitsTool": "DG-FRAN-04", "DraftPickResetTool": "DG-FRAN-06", "FaTrimTool": "DG-FRAN-05", "FranchiseView": "DG-FRAN-03", "HistoricSeasonTool": "DG-FRAN-10",
 "PlayerBulkTools": "DG-FRAN-02", "RelocationTool": "DG-FRAN-08", "ScheduleViewer": "DG-FRAN-07", "GearEditor": "DG-APP-02", "HomePage": "DG-SHELL-01", "MenuBar": "DG-SHELL-01", "MetaStrip": "DG-DATA-03", "OpenClass": "DG-FILE-02;DG-FILE-03",
 "PlayerCards": "DG-BRD-02", "PlayerEditPanel": "DG-EDT-02", "PlayerSearch": "DG-EDT-01", "PlayerTable": "DG-BRD-01", "PositionBreakdown": "DG-BRD-01", "ProfileModal": "DG-EDT-01", "RadarChart": "DG-EDT-01", "RandomDraft": "DG-CLS-02",
 "RosterEditor": "DG-FRAN-03;DG-ROS-02", "RosterBuilder": "DG-ROS-03", "RosterPicker": "DG-ROS-02", "RostersView": "DG-ROS-02", "TeamPanel": "DG-ROS-02", "SideRail": "DG-SHELL-01", "TeamPicker": "DG-CLS-05", "Toolbar": "DG-BRD-01",
 "TopBar": "DG-SHELL-01", "UpdateBanner": "DG-SHELL-03", "WallBoard": "DG-BRD-02", "WhatsNew": "DG-SHELL-02", "YearPicker": "DG-DATA-01",
 "shared": None, "ui": None, "VirtualList": None, "GamePicker": "NA",
}
for c in DG["react_components"]:
    n = os.path.basename(c)[:-4]
    if n not in DC:
        errors.append("unmapped draft component " + c)
        continue
    m = DC[n]
    if m is None:
        sym("SYM-DG-C-" + c, "Draft generator", DG_V, "component", c, "shared", "", c, "code read", "UI primitive.")
    elif m == "NA":
        sym("SYM-DG-C-" + c, "Draft generator", DG_V, "component", c, "out-of-scope", "", c, "code read", "M26/M27 picker: Madden 27 only.")
    else:
        sym("SYM-DG-C-" + c, "Draft generator", DG_V, "component", c, "feature", m, c, "code read")

DS = {
 "ArchetypeService": "DG-RATE-06", "AttributeModel": "DG-RATE-06", "AwardsService": "DG-DATA-05", "CalibrationService": "DG-RATE-02", "ClassName": "DG-CLS-12", "CombineService": "DG-RATE-06", "CuratedDbPositions": "DG-DATA-05",
 "CuratedGearService": "DG-APP-02", "CuratedSkinToneService": "DG-APP-05", "CustomPortraitIdService": "DG-PACK-01", "DbPositionService": "DG-DATA-05", "DerivedSkinToneService": "DG-APP-05", "DevTraitService": "DG-RATE-07",
 "DraftClassBuilder": "DG-CLS-01", "DraftEnrichment": "DG-CLS-01", "EraBioService": "DG-CLS-09", "EraGearService": "DG-APP-02", "EraRulesService": "DG-CLS-09", "FaceFeatures": "DG-APP-01", "FranchiseService": "DG-FRAN-01;DG-FRAN-02;DG-FRAN-03;DG-FRAN-04;DG-FRAN-05;DG-FRAN-06;DG-FRAN-07;DG-FRAN-08;DG-FRAN-09",
 "FrontSevenClassifier": "DG-DATA-05", "FrontSevenService": "DG-DATA-05", "GearImageService": "DG-APP-02", "GearOptionsService": "DG-APP-02", "GenericFillerService": "DG-CLS-09", "HistoricalAccoladeService": "DG-DATA-05",
 "HistoricFranchiseService": "DG-FRAN-10", "HometownService": "DG-CLS-09", "LaunchRatingsService": "DG-RATE-01", "LeagueHistoryData": "DG-FRAN-11", "LeagueHistoryService": "DG-FRAN-11", "LikenessOverrideService": "DG-APP-04", "LikenessService": "DG-APP-04",
 "LogoService": "DG-APP-06", "LookupService": "DG-EDT-02", "M27Fields": "DG-RATE-09", "M27RookieRatingsService": "DG-RATE-01", "Mdc27Service": "DG-FILE-01", "NflverseCareerService": "DG-DATA-03", "NflverseStatsService": "DG-DATA-03",
 "OpenedClassService": "DG-FILE-02", "OVRWeightsCalculator": "DG-RATE-06", "PersonaService": "DG-RATE-08", "PhotoLookService": "DG-APP-03", "PlayerLookupService": "DG-DATA-01;DG-DATA-02", "PoolCatalogService": "DG-CLS-06", "PortraitFetchService": "DG-PACK-05",
 "PortraitModService": "DG-PACK-07", "PortraitPackService": "DG-PACK-01;DG-PACK-02;DG-PACK-03;DG-PACK-04;DG-PACK-05;DG-PACK-06", "PortraitService": "DG-APP-01;DG-APP-06", "PortraitSlotService": "DG-PACK-07", "PositionLabel": "DG-RATE-06", "PositionMapper": "DG-RATE-06",
 "RatingService": "DG-RATE-02;DG-RATE-03", "RealPlayerGearService": "DG-APP-02", "RecomputeService": "DG-RATE-03", "ReferenceDraftService": "DG-DATA-05", "RetroHeadshotService": "DG-APP-06;DG-PACK-05", "RetroItaService": "DG-APP-06", "RosterAddService": "DG-ROS-04",
 "RosterBuildService": "DG-ROS-03", "RosterContainer": "DG-ROS-01", "RosterFileService": "DG-ROS-01", "RosterPositionService": "DG-ROS-01", "SchemeService": "DG-CLS-01", "ScoutingNotesService": "DG-BRD-04", "SeasonPackService": "DG-FRAN-10", "SkinToneClassify": "DG-APP-05",
 "SkinToneService": "DG-APP-05", "SupplementalDraftService": "DG-DATA-05", "Tdb2Engine": "DG-ROS-01", "TeamDraftService": "DG-DATA-05", "TeamService": "DG-DATA-05", "TwoWayService": "DG-DATA-05", "WikipediaTeamService": "DG-DATA-05", "WikiSkinToneService": "DG-APP-05",
}
for s in DG["services"]:
    n = s[:-3]
    if n == "MdcService":
        sym("SYM-DG-S-" + s, "Draft generator", DG_V, "service", s, "out-of-scope", "", "server/src/services/" + s, "code read", "M26 codec: useful only as a refusal fixture.")
    elif n not in DS:
        errors.append("unmapped draft service " + s)
    else:
        sym("SYM-DG-S-" + s, "Draft generator", DG_V, "service", s, "feature", DS[n], "server/src/services/" + s, "code read")

# integrity
feature_ids = {r["id"] for r in rows if r["row_type"] == "feature"}
for r in rows:
    for ref in filter(None, (r["maps_to"] + ";" + r["dependencies"]).split(";")):
        if ref not in feature_ids:
            errors.append(f"{r['id']} references unknown {ref}")
    if r["row_type"] == "feature":
        for k in ("evidence", "observable_behavior", "acceptance_test", "work_package", "runtime_evidence"):
            if not r[k]:
                errors.append(f"{r['id']} missing {k}")
mapped = {x for r in rows for x in r["maps_to"].split(";") if x}
orphans = sorted(i for i in feature_ids if not i.startswith(("XP-", "F-")))
if errors:
    print("\n".join(errors))
    sys.exit(1)
out = os.path.join(DOCS, "FEATURE-PARITY.csv")
with open(out, "w", newline="", encoding="utf-8-sig") as f:
    w = csv.DictWriter(f, COLUMNS, quoting=csv.QUOTE_MINIMAL)
    w.writeheader()
    w.writerows(rows)
from collections import Counter
print("rows", len(rows), "features", len(feature_ids))
print("by source", Counter(r["source"] for r in rows if r["row_type"] == "feature"))
print("symbols", Counter((r["symbol_kind"], r["classification"]) for r in rows if r["row_type"] == "symbol"))
print("features with no direct symbol map (behaviour-derived):", len([i for i in orphans if i not in mapped]))
print("wp", sorted(Counter(r["work_package"] for r in rows if r["row_type"] == "feature").items()))
