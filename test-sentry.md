# Quick Sentry Test Instructions

## The Issue
The React frontend Sentry is working ✅, but the Rust backend Sentry is not sending events ❌.

## Why?
The `SENTRY_DSN` is already configured in `.cargo/config.toml`, but the running app was compiled **before** this configuration was added. The Rust code reads `SENTRY_DSN` at compile time, not runtime.

## Solution

### Step 1: Stop the current dev server
Press `Ctrl+C` in the terminal where `pnpm tauri dev` is running.

### Step 2: Restart the dev server
```bash
pnpm tauri dev
```

This will recompile the Rust code with the `SENTRY_DSN` environment variable included.

### Step 3: Check diagnostics are enabled
1. Open WhisprTypr settings
2. Find "Enable diagnostics" or similar setting
3. Make sure it's **turned ON** (Sentry only sends events when diagnostics are enabled)

### Step 4: Test again
1. Go to **Help & Support** in the app
2. Scroll to **Developer Tools** section
3. Click **Test Rust Backend** button
4. Check your [Sentry dashboard](https://sentry.io)

## Expected Result
You should see a new issue in Sentry with the message:
> "Sentry test from Rust backend - this is a test message"

## Still Not Working?

### Check the terminal logs
When the app starts, you should see:
```
[INFO whisprtypr_lib] Sentry initialized
```

If you don't see this message, it means either:
1. `SENTRY_DSN` wasn't compiled in (needs rebuild)
2. Diagnostics are disabled in settings

### Verify SENTRY_DSN is compiled
Run this in the terminal after stopping the app:
```bash
cd src-tauri
cargo clean
cargo build
```

Then restart: `pnpm tauri dev`

## Alternative: Manual Test via Error Reporting

You can also trigger a backend Sentry event by using the error reporting command.

Open browser console (F12) and run:
```javascript
await window.__TAURI__.core.invoke('report_error', {
  category: 'system',
  message: 'Manual test of Rust Sentry',
  severity: 'error'
});
```

This should appear in Sentry if backend integration is working.
