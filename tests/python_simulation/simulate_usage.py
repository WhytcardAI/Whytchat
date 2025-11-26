import subprocess
import time
import pyautogui
import pygetwindow as gw
import os
import sys
import psutil

# Configuration
APP_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../target/debug/whytchat-core.exe"))
WINDOW_TITLE = "WhytChat V1"
LOG_FILE = "simulation_log.txt"

def log(message):
    timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{timestamp}] {message}")
    with open(LOG_FILE, "a") as f:
        f.write(f"[{timestamp}] {message}\n")

def is_app_running(process_name="whytchat-core.exe"):
    for proc in psutil.process_iter(['name']):
        if proc.info['name'] == process_name:
            return True
    return False

def main():
    log("Starting simulation...")

    # 1. Check if app exists
    if not os.path.exists(APP_PATH):
        log(f"Error: App executable not found at {APP_PATH}")
        log("Please run 'npm run tauri build' or 'cargo build' first.")
        return

    # 2. Launch App
    log(f"Launching app from {APP_PATH}...")
    process = subprocess.Popen([APP_PATH])

    # 3. Wait for Window
    log("Waiting for window...")
    window = None
    for i in range(30): # Wait up to 30 seconds
        windows = gw.getWindowsWithTitle(WINDOW_TITLE)
        if windows:
            window = windows[0]
            break
        time.sleep(1)

    if not window:
        log("Error: Window not found after 30 seconds.")
        process.terminate()
        return

    log("Window found! Bringing to front...")
    try:
        window.activate()
    except:
        log("Could not activate window (might already be active).")

    time.sleep(2) # Wait for UI to load

    # 4. Simulate Interaction
    log("Simulating user interaction...")

    # Type a message
    log("Typing message...")
    pyautogui.write("Hello from Python Simulation!", interval=0.1)
    time.sleep(1)

    # Press Enter
    log("Sending message...")
    pyautogui.press('enter')

    # Wait for response (simulation)
    log("Waiting for response (simulated wait)...")
    time.sleep(5)

    # 5. Take Screenshot
    screenshot_path = "simulation_screenshot.png"
    log(f"Taking screenshot to {screenshot_path}...")
    try:
        screenshot = pyautogui.screenshot()
        screenshot.save(screenshot_path)
        log("Screenshot saved.")
    except Exception as e:
        log(f"Failed to take screenshot: {e}")

    # 6. Cleanup
    log("Closing application...")
    window.close()

    # Ensure process is dead
    time.sleep(2)
    if is_app_running():
        log("Force killing process...")
        process.kill()

    log("Simulation complete.")

if __name__ == "__main__":
    main()
