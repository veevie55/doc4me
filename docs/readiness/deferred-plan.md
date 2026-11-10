# Deferred readiness: active work and verification

Reviewed 2026-11-07. The owner explicitly resumed all deferred work. The 22 rows remain
incomplete until implementation and the stated verification succeed. Original TSV notes that
say “skip until asked” describe the scheduling history, not the current blocker. No row is
marked passing by this plan. The last official measurement remains Windows run 42836913553.

## Dialogs (12)

| Row | Remaining implementation / prerequisite | Completion evidence |
|---|---|---|
| dlg-002 | Windows network-drive host hook and permitted test SMB share | Connect/disconnect test share, open/save DOCX; no credentials in logs |
| dlg-005 | User-initiated Windows Control Panel host hook | Native UI opens from user action; CLI/MCP never opens it |
| dlg-006 | Safe native OLE conversion bridge and installed convertible OLE server | Convert fixture class, undo, save/reopen; payload preservation alone is insufficient |
| dlg-016 | Legacy Mac publish-and-subscribe runtime | Create interoperable publisher fixture; unavailable on the current Linux/modern Windows hosts |
| dlg-022 | Safe OLE activation/editing bridge and installed server for the object's ProgID | User edits embedded fixture, bytes/preview survive save/reopen and undo |
| dlg-024 | Legacy Mac publisher-options runtime | Change publisher options and verify saved behavior |
| dlg-027 | Legacy Mac subscriber-options runtime | Change subscription options and verify updates from publisher |
| dlg-028 | Legacy Mac subscribe-to runtime and publisher/subscriber pair | Subscribe, update and save/reopen; ordinary file links are insufficient |
| dlg-030 | Compatible Outlook/MAPI profile and native integration | Open and persist stationery/signature options; mail merge is separate |
| dlg-042 | Compatible legacy Exchange/MAPI routing provider and integration | Verify routing slip with test recipients; sending needs separate explicit authorization |
| dlg-084 | Windows IME settings host hook and installed IME | Open settings, change/restore test profile default, compose and commit |
| dlg-113 | Compatible licensed Office Web Component and ActiveX host/serialization | Insert, interact, save/reopen actual component; preserving XML is insufficient |

Application code forbids unsafe Rust. Native bridges need audited safe interfaces, explicit
user initiation, cancellation/error handling and bounded arguments. Do not automatically
activate untrusted embedded payloads to obtain a passing check.

## OS input (4)

| Row | Remaining prerequisite | Completion evidence on Windows |
|---|---|---|
| ks-173 | Numeric keypad and Windows Alt-code conversion | Alt+0128 inserts exactly one euro sign; test body, box and dialog fields, retaining ribbon keys |
| ks-245 | Japanese Microsoft IME with 101-key keyboard | Alt+backquote switches mode; candidate commit/cancel produces no duplicated body/box text |
| ks-246 | Korean Microsoft IME | Right Alt switches Hangul/Latin; compositions commit once, including in selected text boxes |
| ks-247 | Chinese Microsoft Pinyin | Ctrl+Space switches configured mode; commit/cancel preserves text in body, box and equation |

This round fixes IME commit routing into a selected text box. Synthetic regressions prove
that its body anchor survives; they do not establish the Windows switching checks above.

## Ecosystem and hardware (6)

| Row | Remaining implementation / prerequisite | Completion evidence |
|---|---|---|
| eco-005 | Mendeley client integration, application registration, public-client OAuth flow and authorized test library | Insert two citations, change style, refresh bibliography, save/reopen, update library item, refresh and undo; cover denied/expired auth and document replacement |
| hw-001 | Native printer-options implementation and physical Windows printer; current code opens a PDF viewer | Select printer/copies/duplex/page range; verify output plus cancel/error recovery |
| hw-003 | Two physical Windows monitors with distinct DPI | Move/reopen, change scaling, disconnect/reconnect; window/dialog/caret geometry stays visible and correct |
| hw-004 | Pressure sample pipeline through model/engine/render/export and pressure-capable pen; current InkDrag records XY only | Light/heavy strokes, eraser, undo, save/reopen and PDF output retain pressure behavior |
| hw-005 | Physical Windows microphone and permission | Dictate known phrase, pause/stop/restart; deny permission/unplug and recover without duplicated/lost text |
| hw-007 | Native WIA/TWAIN or camera acquisition adapter and physical device | Acquire image, cancel/error recovery, insert and save/reopen; file insertion alone is insufficient |

Mendeley public-catalog access cannot prove personal-library integration. Its [authorization
documentation](https://dev.mendeley.com/reference/topics/authorization_overview.html) requires
application registration/OAuth; no account was registered or secret embedded in this session.
Hosting the actual Mendeley Cite add-in would additionally require an Office.js-compatible host.

The current Linux shell has no Windows runtime, physical sound/input/video devices or Windows
IME profiles. A hosted Windows build runner does not supply the required personal hardware or
legacy servers. Record app commit, Windows build, device/driver or service version, input
language/layout, procedure, actual result and date before updating a TSV status. Baseline OLE
preservation, embedded-relationship safety and citation-recognition tests pass; these do not
prove native OLE activation, ActiveX or authenticated Mendeley integration.
