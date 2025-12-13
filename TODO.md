# 🚀 Next Steps for Cyber Forum

## Immediate Tasks
- **Deploy the server** to a remote host (VPS, Docker, etc.).
- **Add TLS**: front the Actix server with Nginx/Caddy for HTTPS.
- **Improve authentication**: replace the simple `hash_` scheme with bcrypt + JWT.
- **Error handling**: surface API errors in the TUI (e.g., toast messages).

## Medium‑Term Enhancements
- **Web front‑end**: reuse the same API from a React/Vue SPA.
- **Rich text**: support markdown rendering for thread content & comments.
- **Search & pagination**: API endpoints for filtered thread lists.
- **User profiles**: view/edit user info, avatar support.

## Long‑Term Vision
- **Federated forums**: multiple server instances syncing via a central hub.
- **Real‑time updates**: WebSocket or Server‑Sent Events for live comment streams.
- **Theming**: dark/light mode, custom colors for the TUI.

## 🔒 Secret Club Requirements
- **Omarchy Exclusive**: Implement strict User-Agent or OS-fingerprinting checks to ensure ONLY users running **Omarchy** can access the forum. No other Arch flavors, no other distros.

## 📦 Installation & Distribution
- **`install.sh` Script**: Create a one-step install script (`clone` -> `cd` -> `./install.sh`).
- **App Launcher Integration**: Generate a `.desktop` file so it appears in the system menu.
- **Global Hotkey**: Configure `Super+Shift+P` to instantly launch the forum (needs DE/WM detection or manual instructions).
