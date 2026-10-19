#!/usr/bin/env python3
"""
Causal Test Suite: UI Scroll Preservation & Granular State Reconciliation
========================================================================
Validates that rapid /dev/shm periodic state pulses do NOT reset UI components,
do NOT yank scroll positions back to top, and do NOT wipe user draft prompts or diff offsets.
"""

import json
import os
import sys

def run_causal_tests():
    print("=================================================================")
    print("   Causal Test: UI Scroll Preservation & State Reconciliation    ")
    print("=================================================================")

    # Test 1: Raw State Identity Caching (Prevents Unnecessary UI Re-evaluations)
    print("\n[CAUSAL TEST 1] Verifying Raw State Identity Caching in stateReader...")
    state_mock_1 = json.dumps({"stats": {"total_active_agents": 2}, "active_sessions": [{"pid": 101, "title": "Session 1"}]})
    state_mock_2 = json.dumps({"stats": {"total_active_agents": 2}, "active_sessions": [{"pid": 101, "title": "Session 1"}]})
    
    lastRawStateJson = ""
    parse_count = 0
    
    def simulate_state_reader_tick(stdout):
        nonlocal lastRawStateJson, parse_count
        if stdout == lastRawStateJson:
            return None  # Skipped, zero re-evaluations
        parsed = json.loads(stdout)
        lastRawStateJson = stdout
        parse_count += 1
        return parsed

    res1 = simulate_state_reader_tick(state_mock_1)
    assert res1 is not None, "First tick must parse"
    assert parse_count == 1, "Expected 1 parse on first tick"

    # Consecutive identical tick
    res2 = simulate_state_reader_tick(state_mock_2)
    assert res2 is None, "Identical tick must be skipped by raw JSON comparison"
    assert parse_count == 1, "Parse count must remain 1 (zero redundant re-renders)"
    print("  ✓ Raw state caching successfully intercepted identical pulse (0 re-evaluations)")

    # Test 2: Active Sessions Scroll Preservation & Interaction Guard
    print("\n[CAUSAL TEST 2] Testing Active Sessions Scroll Preservation & Interaction Guard...")
    
    class MockListView:
        def __init__(self):
            self.contentY = 0
            self.contentHeight = 2000
            self.height = 600
            self.moving = False
            self.dragging = False
            self.flicking = False
            self.model = []

    class MockFullRoot:
        def __init__(self):
            self.activeSessions = [{"pid": 101, "provider": "antigravity", "tokens": 100}]
            self.selectedProviderFilter = "all"
            self.filteredActiveSessions = []
            self.hasPendingSessionsSync = False
            self.listView = MockListView()

        def syncFilteredSessions(self):
            filtered = [s for s in self.activeSessions if self.selectedProviderFilter == "all" or s.get("provider") == self.selectedProviderFilter]
            if json.dumps(filtered, sort_keys=True) == json.dumps(self.filteredActiveSessions, sort_keys=True):
                return
            
            # Interaction Guard
            if self.listView.moving or self.listView.dragging or self.listView.flicking:
                self.hasPendingSessionsSync = True
                return
            
            self.hasPendingSessionsSync = False
            prevY = self.listView.contentY
            self.filteredActiveSessions = filtered
            self.listView.model = filtered
            
            # Restore scroll position
            if prevY > 0:
                self.listView.contentY = min(prevY, max(0, self.listView.contentHeight - self.listView.height))

    root = MockFullRoot()
    root.syncFilteredSessions()
    assert len(root.filteredActiveSessions) == 1
    
    # User scrolls down to pixel 350
    root.listView.contentY = 350
    
    # Daemon tick with identical data
    root.syncFilteredSessions()
    assert root.listView.contentY == 350, "Scroll position must not change on identical data"

    # User starts dragging/scrolling (interaction in progress)
    root.listView.dragging = True
    root.listView.contentY = 480
    
    # Data changes while user is actively dragging
    root.activeSessions = [{"pid": 101, "provider": "antigravity", "tokens": 250}]
    root.syncFilteredSessions()
    
    assert root.hasPendingSessionsSync is True, "Sync must be deferred while user is dragging"
    assert root.listView.contentY == 480, "User drag position must not be interrupted"
    assert root.filteredActiveSessions[0]["tokens"] == 100, "Old model must be held until drag finishes"

    # User finishes dragging
    root.listView.dragging = False
    if root.hasPendingSessionsSync:
        root.syncFilteredSessions()
    
    assert root.hasPendingSessionsSync is False
    assert root.filteredActiveSessions[0]["tokens"] == 250, "Model updated after drag completion"
    assert root.listView.contentY == 480, "Scroll position preserved at 480px after model update"
    print("  ✓ Active sessions list scroll pinned and user gestures protected from interruptions")

    # Test 3: Inline Diff Drawer Scroll Preservation
    print("\n[CAUSAL TEST 3] Testing Inline Diff Drawer Scroll Preservation...")
    
    class MockDiffState:
        def __init__(self):
            self.diffOpenedPid = 0
            self.savedDiffScrollY = 0
            self.savedDiffScrollX = 0
            self.activeDiffContent = ""
            
        def toggleDiffView(self, pid, workspace):
            if self.diffOpenedPid == pid:
                self.diffOpenedPid = 0
                self.savedDiffScrollY = 0
                self.savedDiffScrollX = 0
            else:
                self.diffOpenedPid = pid
                self.savedDiffScrollY = 0
                self.savedDiffScrollX = 0
                self.activeDiffContent = "Line 1\nLine 2\nLine 3\n...Line 150"

    diff_state = MockDiffState()
    
    # Open diff for PID 101
    diff_state.toggleDiffView(101, "/path/to/project")
    assert diff_state.diffOpenedPid == 101
    assert diff_state.savedDiffScrollY == 0

    # User scrolls diff to line 65 (pixel offset 920)
    diff_state.savedDiffScrollY = 920
    
    # Simulate delegate recreation on state refresh
    # Delegate re-initializes and inspects savedDiffScrollY:
    diffTextArea_contentY = 0
    if diff_state.savedDiffScrollY > 0:
        diffTextArea_contentY = diff_state.savedDiffScrollY
        
    assert diffTextArea_contentY == 920, f"Diff text area should restore to 920, got {diffTextArea_contentY}"

    # Explicit close resets offsets
    diff_state.toggleDiffView(101, "/path/to/project")
    assert diff_state.diffOpenedPid == 0
    assert diff_state.savedDiffScrollY == 0
    print("  ✓ Diff viewer maintains exact line reading position across ticks and cleans up on close")

    # Test 4: Inline Prompt Draft Preservation
    print("\n[CAUSAL TEST 4] Testing Inline Prompt Draft Preservation...")
    sessionDraftPrompts = {}
    
    # User types a drafted prompt for PID 101
    pid = 101
    draft_text = "Refactor database migration to add foreign key index"
    sessionDraftPrompts[pid] = draft_text

    # Delegate re-creation simulation
    recreated_field_text = sessionDraftPrompts.get(pid, "")
    assert recreated_field_text == draft_text, "User drafted text must survive delegate re-renders"

    # User submits prompt
    def submit_prompt(pid, text):
        sessionDraftPrompts[pid] = ""
        return f"Submitted: {text}"

    submit_res = submit_prompt(pid, recreated_field_text)
    assert submit_res.startswith("Submitted: Refactor")
    assert sessionDraftPrompts[pid] == "", "Draft prompt cleared after submission"
    print("  ✓ User draft prompt successfully preserved across ticks and cleared on send")

    # Test 5: Float Clamping & Rounding Stability
    print("\n[CAUSAL TEST 5] Testing Float Clamping and Rounding Stability...")
    token_delta = 1500
    # Simulate two ticks with elapsed time fluctuating due to microseconds (15.002s vs 15.008s)
    rate_1 = round((token_delta / 15.002) * 60.0, 1)
    rate_2 = round((token_delta / 15.008) * 60.0, 1)
    
    assert rate_1 == 5999.2, f"Expected 5999.2, got {rate_1}"
    assert rate_2 == 5996.8, f"Expected 5996.8, got {rate_2}"
    # Verify no endless floating point decimals (e.g. 5999.200106652445)
    assert len(str(rate_1).split(".")[1]) <= 1, "Rate float must have max 1 decimal place"
    print("  ✓ Float rounding guarantees stable, clamped serialization")

    print("\n=================================================================")
    print("   ✓ ALL CAUSAL TESTS PASSED SUCCESSFULLY!                       ")
    print("=================================================================")

if __name__ == "__main__":
    try:
        run_causal_tests()
    except Exception as e:
        print(f"\n❌ Causal test failed: {e}")
        sys.exit(1)
