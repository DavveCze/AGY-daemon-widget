#!/usr/bin/env python3
import json
import os
import subprocess
import sys

def test_metadata():
    print("[TEST] Validating metadata.json...")
    meta_path = os.path.expanduser("~/Projekty/AGY_Widget/plasmoid/org.kde.plasma.antigravity/metadata.json")
    assert os.path.exists(meta_path), f"File {meta_path} does not exist"
    
    with open(meta_path, "r", encoding="utf-8") as f:
        meta = json.load(f)
        
    assert meta.get("KPackageStructure") == "Plasma/Applet", "Must be Plasma/Applet"
    kplugin = meta.get("KPlugin", {})
    assert kplugin.get("Id") == "org.kde.plasma.antigravity", "Incorrect KPlugin Id"
    assert "Name" in kplugin, "Missing KPlugin Name"
    assert meta.get("X-Plasma-API-Minimum-Version") == "6.0", "Must target Plasma 6.0+"
    print("  ✓ metadata.json is valid Plasma 6 configuration")

def test_qmllint():
    print("[TEST] Running qmllint on QML files...")
    qmllint_bin = "/usr/lib/qt6/bin/qmllint"
    if not os.path.exists(qmllint_bin):
        print("  ! qmllint not found, skipping linting")
        return

    qml_dir = os.path.expanduser("~/Projekty/AGY_Widget/plasmoid/org.kde.plasma.antigravity/contents/ui")
    files = [
        os.path.join(qml_dir, "main.qml"),
        os.path.join(qml_dir, "CompactRepresentation.qml"),
        os.path.join(qml_dir, "FullRepresentation.qml"),
    ]
    for qf in files:
        assert os.path.exists(qf), f"{qf} not found"
        res = subprocess.run([qmllint_bin, qf], capture_output=True, text=True)
        # qmllint returns non-zero only on fatal syntax errors
        assert res.returncode == 0, f"qmllint failed for {qf}:\n{res.stderr}\n{res.stdout}"
        print(f"  ✓ {os.path.basename(qf)} passed syntax validation")

def test_kpackagetool():
    print("[TEST] Checking kpackagetool6 registration...")
    res = subprocess.run(["kpackagetool6", "--type", "Plasma/Applet", "--list"], capture_output=True, text=True)
    assert res.returncode == 0
    assert "org.kde.plasma.antigravity" in res.stdout, "Widget not found in kpackagetool6 --list"
    print("  ✓ Plasmoid successfully recognized by KDE Plasma 6 package manager")

def test_shm_state():
    print("[TEST] Verifying /dev/shm/agy-state.json...")
    shm_path = "/dev/shm/agy-state.json"
    if not os.path.exists(shm_path):
        print("  ! Notice: /dev/shm/agy-state.json not yet written by daemon (will test dump instead)")
        dump_res = subprocess.run(["~/.local/bin/agy-daemon", "dump"], capture_output=True, text=True, shell=True)
        data = json.loads(dump_res.stdout)
    else:
        with open(shm_path, "r", encoding="utf-8") as f:
            data = json.load(f)

    assert "stats" in data, "Missing 'stats' in state"
    assert "active_sessions" in data, "Missing 'active_sessions' in state"
    assert "settings" in data, "Missing 'settings' in state"
    assert "skills" in data, "Missing 'skills' in state"
    print(f"  ✓ State verified: {data['stats']['total_active_agents']} active sessions, {len(data['skills'])} skills detected")

if __name__ == "__main__":
    print("=== Running Antigravity Plasmoid & UI Test Suite ===")
    try:
        test_metadata()
        test_qmllint()
        test_kpackagetool()
        test_shm_state()
        print("\nAll UI & Plasmoid tests passed successfully!")
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        sys.exit(1)
