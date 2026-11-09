# FleetManager

Applicazione desktop nativa per la gestione di una flotta aziendale: veicoli, autisti, assegnazioni, manutenzioni, rifornimenti, spese e scadenze dei documenti. Scritta in Rust con [egui](https://github.com/emilk/egui), funziona **offline** e salva tutti i dati in locale in un database SQLite.

## Funzionalità

- **Dashboard** con statistiche, grafici, scadenze imminenti e attività recenti
- **Veicoli**: anagrafica, immagine, stato, acquisizione e chilometraggio
- **Autisti**: dati anagrafici, patente e scadenza, tipo di impiego
- **Assegnazioni** veicolo ↔ autista con km di inizio e fine
- **Manutenzioni** programmate e completate, con costi e prossime scadenze
- **Carburante e spese** per veicolo
- **Documenti** di veicoli e autisti con allegati e avvisi di scadenza
- **Report** esportabili in **PDF** e **CSV**
- **Accesso con utenti**: login, ruolo amministratore, cambio password obbligatorio al primo accesso
- **Backup e ripristino** del database
- Interfaccia in **italiano** e **inglese**, tema chiaro/scuro, valuta e formato data configurabili

## Accesso e utenti

Al primo avvio viene creato l'utente `admin` con una **password temporanea casuale**:

- viene mostrata nella schermata di login e salvata in `admin_initial_password.txt` nella cartella dei dati (permessi `600`, si cancella da sola dopo il cambio password);
- al primo accesso l'app richiede di scegliere una nuova password (almeno 8 caratteri) prima di continuare.

Gli amministratori possono, da **Impostazioni → Utenti**, creare altri utenti, reimpostare le password, disabilitare/abilitare ed eliminare gli account. Ogni nuovo utente (o password reimpostata) riceve una password temporanea casuale, mostrata una sola volta, che deve essere cambiata al primo accesso.

Note di sicurezza:

- le password sono salvate con hash **Argon2**, mai in chiaro;
- dopo 5 tentativi errati il login si blocca per 30 secondi;
- non si può disabilitare né eliminare l'ultimo amministratore attivo;
- backup e ripristino sono riservati agli amministratori.

## Compilazione (Linux)

Requisiti: [Rust](https://rustup.rs) (stable recente) e le librerie di sistema per finestra e dialoghi.

Debian/Ubuntu:

```bash
sudo apt install build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libgtk-3-dev
```

Fedora:

```bash
sudo dnf install gcc pkg-config libxcb-devel libxkbcommon-devel gtk3-devel openssl-devel
```

Arch:

```bash
sudo pacman -S base-devel pkgconf libxcb libxkbcommon gtk3
```

Poi:

```bash
cargo build --release
./target/release/fleetmanager
```

Il binario è un unico file e può essere copiato dove vuoi. SQLite è incluso nel binario, non serve installarlo. La prima compilazione richiede qualche minuto (LTO attivo nel profilo release).

> Su Wayland, in caso di problemi con la finestra: `WINIT_UNIX_BACKEND=x11 ./target/release/fleetmanager`

## Dove vengono salvati i dati

| Sistema | Percorso |
|---|---|
| Linux | `~/.local/share/FleetManager/` |
| Windows | `%APPDATA%\FleetManager\` |

La cartella contiene `fleetmanager.db` (database) e `attachments/` (allegati dei documenti). Il percorso esatto è mostrato in **Impostazioni → Database**.

## Struttura del progetto

```
src/
├── main.rs        avvio e finestra
├── app.rs         stato dell'app, login e navigazione
├── db/            database SQLite (veicoli, autisti, utenti, report, ...)
├── ui/            interfaccia egui (pagine, login, widget, tema, grafici)
├── i18n/          traduzioni (inglese → italiano)
└── util/          formati, CSV, PDF, file e input
```

## Test

```bash
cargo test
```

## Stack

[Rust](https://www.rust-lang.org) · [eframe/egui](https://github.com/emilk/egui) · [rusqlite](https://github.com/rusqlite/rusqlite) (SQLite) · [argon2](https://crates.io/crates/argon2) · [chrono](https://crates.io/crates/chrono) · [rfd](https://github.com/PolyMeilex/rfd)