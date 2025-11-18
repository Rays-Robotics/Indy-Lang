# Indy-Lang  
**I.N.D.Y. = I’m Not Doing YAML**

**WARNING: THIS IS IN EARLY DEVELOPMENT AND CANNOT DO A LOT OF THINGS YET. USE AT YOUR OWN RISK.**

Indy-Lang is a blazing fast, ultra-simple programming language built on Rust. It’s readable, fun, and powerful enough for beginners, pros, kids, and anyone who just wants to get stuff done without wrestling with complicated syntax.

---

## Why Indy-Lang?

- **Fast:** Rust under the hood means scripts run lightning quick.
- **Readable:** Commands like `say`, `prompt`, and `wait` read like English.
- **Plugs:** Extend Indy-Lang with “plugs” (plugins) easily.
- **GUI + CLI:** Use the terminal or the Indy Editor (coming soon) to create, edit, and run scripts.
- **Truly FOSS:** Licensed under **GPLv3**, ensuring your code and modifications always remain free.

---

## Example Script

```indy
start

prompt Name="Hello! What's your name?"

say "Thanks {Name}! Welcome to Indy-Lang."

wait 2

say "You can print, loop (WIP), prompt, and even make web apps (WIP)."

end
````

This script shows the basics: prompting, printing, waiting, and variable interpolation.

---

## Getting Started

### Using the CLI

Install Indy-Lang and run scripts like this:

```bash
indy main.indy
```

### Using the GUI Editor (Not yet ready, Everything in this part is either very early in development, Or development has not started for it yet.)

* Open Indy Editor (coming soon)
* Press “Get New Plug” to browse and install plugs
* Create, edit, and run Indy scripts with a single run button

---

## Plugs (Not yet ready, Everything in this part is either very early in development, Or development has not started for it yet.)

Plugs allow you to extend Indy-Lang without breaking simplicity.

Examples:

```indy
plug import GUI        # Create GUI applications
plug import localhost  # Run a web server
```

Install via CLI:

```bash
indy plug install GUI
```

Install via GUI: click “Get New Plug”, browse the catalog, and press install.

---

## License

Indy-Lang is licensed under **GPLv3**. This means:

* All scripts, modifications, and plugs must remain free when redistributed
* No proprietary forks
* Patents and “tivoization” cannot restrict use

---

## Vision

Indy-Lang is designed to be **accessible, readable, and powerful for everyone**. The goal is to eventually handle everything Python can do, but faster, simpler, and more fun.

Planned future features include:

* Scripts
* Web apps
* GUI apps
* Full Linux distro with custom DE
* A fully integrated ecosystem of plugs

Everything will be usable through CLI or GUI.

---

## Fun Feature (ok i have no idea why i have not put this in yet but i really should)

Indy-Lang comes with a dancing parrot in the terminal (like `curl parrot.live`) to make learning and using the language more fun.

---

## Get Involved

* Try Indy-Lang today
* Create and share plugs (Nah you cant do they yet, Sorry.)
* Support development through donations (Nah you cant do they yet, Sorry.)
* Contribute to the growing ecosystem

Documentation and full guides are coming soon at `indy.raysrobotics.com/docs`.
