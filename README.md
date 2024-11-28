# Zellij Favs a plugin for [Zellij](https://github.com/zellij-org/zellij)

A simple and intuitive plugin for managing favorite sessions in Zellij. With zellij-favs, you can keep your favorite sessions organized and easily flush away the unwanted ones.

# Features

- Filter Sessions: Use / to filter through your sessions quickly.
- Switch Between Lists: Press Tab to toggle between the "Favorites" and "Flush" lists.
- Move Sessions: Select a session and press Space to move it between the lists.
- Flush Unwanted Sessions: Press F to remove all non-favorite sessions.
- Access Sessions: Press Enter, Left Arrow, or Right Arrow to access the highlighted session.

## Usage

- Filtering:
  Press / and start typing to filter sessions.
  After filtering, press Enter to return to the sessions list.

- Switching Lists:
  Press Tab to switch between the "Favorites" and "Flush" lists.

- Managing Sessions:
  Highlight a session and press Space to move it between "Favorites" and "Sessions."

- Flushing Sessions:
  Press F to flush all unwanted sessions from the list.

- Accessing a Session:
  Highlight a session and press Enter to open it.

- Close panel plugin:
  Press Esc to exit the plugin

# Installation

1. Install the plugin using the following command:

```sh
mkdir -p ~/.config/zellij/plugins && \
curl -L https://github.com/JoseMM2002/zellij-favs/releases/download/Latest/zellij-favs.wasm -o ~/.config/zellij/plugins/zellij-favs.wasm
```

2. Add the following keybind to your Zellij [configuration](https://zellij.dev/documentation/configuration.html) file:

```kdl
shared_except "locked" {
    bind "Ctrl {char}" {
        LaunchOrFocusPlugin "file:~/.config/zellij/plugins/zellij-favs.wasm" {
            floating true
            ignore_case true
            quick_jump true
        }
    }
}
```

# Goals

[] Make a plugin that allows users to manage their favorite sessions in Zellij.
[] Keep the plugin data available and synchronized on multiple terminal sessions.
[] Keep the plugin data after reboot

# Contributing

If you have any ideas, feel free to open an issue or a pull request.
This is my first project in Rust, so any feedback is welcome.
Thank you for your support!
