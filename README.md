<div align="center">

# 🚚 FleetManager

**Gestione della flotta aziendale, semplice e offline.**

Veicoli, autisti, manutenzioni, rifornimenti, spese e scadenze in un'unica app desktop nativa.
Nessun cloud, nessun abbonamento: i dati restano sul tuo computer.

![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)
![egui](https://img.shields.io/badge/UI-egui-blue)
![SQLite](https://img.shields.io/badge/DB-SQLite-003B57?logo=sqlite)
![Platform](https://img.shields.io/badge/Linux%20%7C%20Windows-lightgrey)
![License](https://img.shields.io/badge/license-TODO-green)

<!-- TODO: sostituisci con uno screenshot reale -->
<img src="docs/screenshots/dashboard.png" alt="Dashboard di FleetManager" width="800">

</div>

---

## Indice

- [Perché FleetManager](#perché-fleetmanager)
- [Funzionalità](#funzionalità)
- [Avvio rapido](#avvio-rapido)
- [Accesso e utenti](#accesso-e-utenti)
- [Compilazione](#compilazione)
- [Dati e backup](#dati-e-backup)
- [Struttura del progetto](#struttura-del-progetto)
- [Test](#test)
- [Stack tecnologico](#stack-tecnologico)
- [Contribuire](#contribuire)
- [Licenza](#licenza)

## Perché FleetManager

- 🔒 **Privacy**: funziona completamente offline, tutto è salvato in un database SQLite locale.
- ⚡ **Leggero e veloce**: scritto in Rust, distribuito come un singolo binario senza dipendenze runtime (SQLite incluso).
- 🌍 **Pronto all'uso**: interfaccia in italiano e inglese, tema chiaro/scuro, valuta e formato data configurabili.

## Funzionalità

| Area | Cosa puoi fare |
|---|---|
| 📊 **Dashboard** | Statistiche, grafici, scadenze imminenti e attività recenti |
| 🚗 **Veicoli** | Anagrafica, immagine, stato, acquisizione e chilometraggio |
| 👤 **Autisti** | Dati anagrafici, patente e relativa scadenza, tipo di impiego |
| 🔗 **Assegnazioni** | Collegamento veicolo ↔ autista con km di inizio e fine |
| 🔧 **Manutenzioni** | Interventi programmati e completati, costi e prossime scadenze |
| ⛽ **Carburante e spese** | Rifornimenti e spese registrati per veicolo |
| 📄 **Documenti** | Documenti di veicoli e autisti con allegati e avvisi di scadenza |
| 📑 **Report** | Esportazione in **PDF** e **CSV** |
| 🔐 **Utenti** | Login, ruolo amministratore, cambio password obbligatorio al primo accesso |
| 💾 **Backup** | Backup e ripristino del database |

## Avvio rapido

```bash
git clone https://github.com/zNxki/fleet-manager.git
cd fleetmanager
cargo run --release
```

Al primo avvio accedi con l'utente `admin` e la password temporanea mostrata nella schermata di login (vedi [Accesso e utenti](#accesso-e-utenti)).

> Per le dipendenze di sistema su Linux consulta la sezione [Compilazione](#compilazione).

## Accesso e utenti

Al primo avvio viene creato l'utente `admin` con una **password temporanea casuale**:

- viene mostrata nella schermata di login e salvata in `admin_initial_password.txt` nella [cartella dei dati](#dati-e-backup) (permessi `600`); il file si cancella da solo dopo il cambio password;
- al primo accesso l'app richiede di scegliere una nuova password (minimo 8 caratteri) prima di continuare.

Gli amministratori possono, da **Impostazioni → Utenti**, creare altri utenti, reimpostare le password, disabilitare/abilitare ed eliminare gli account. Ogni nuovo utente (o password reimpostata) riceve una password temporanea casuale, mostrata **una sola volta**, che deve essere cambiata al primo accesso.

### Sicurezza

- Le password sono salvate con hash **Argon2**, mai in chiaro.
- Dopo 5 tentativi errati il login si blocca per 30 secondi.
- Non si può disabilitare né eliminare l'ultimo amministratore attivo.
- Backup e ripristino sono riservati agli amministratori.

## Compilazione

**Requisiti:** [Rust](https://rustup.rs) (stable recente) e, su Linux, le librerie di sistema per finestra e dialoghi.

### Linux

<details>
<summary><b>Debian / Ubuntu</b></summary>

```bash
sudo apt install build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libgtk-3-dev
```
</details>

<details>
<summary><b>Fedora</b></summary>

```bash
sudo dnf install gcc pkg-config libxcb-devel libxkbcommon-devel gtk3-devel openssl-devel
```
</details>

<details>
<summary><b>Arch</b></summary>

```bash
sudo pacman -S base-devel pkgconf libxcb libxkbcommon gtk3
```
</details>

Poi:

```bash
cargo build --release
./target/release/fleetmanager
```

### Windows

Installa Rust con il toolchain MSVC (richiede i *Build Tools per Visual Studio*), quindi:

```powershell
cargo build --release
.\target\release\fleetmanager.exe
```

### Note

- Il binario è un **unico file** e può essere copiato dove vuoi. SQLite è incluso, non serve installarlo.
- La prima compilazione richiede qualche minuto (LTO attivo nel profilo release).
- **Wayland**: in caso di problemi con la finestra, avvia con
  ```bash
  WINIT_UNIX_BACKEND=x11 ./target/release/fleetmanager
  ```

## Dati e backup

| Sistema | Percorso |
|---|---|
| Linux | `~/.local/share/FleetManager/` |
| Windows | `%APPDATA%\FleetManager\` |

La cartella contiene:

- `fleetmanager.db`: il database SQLite
- `attachments/`: gli allegati dei documenti

Il percorso esatto è mostrato in **Impostazioni → Database**. Per salvare o ripristinare i dati usa la funzione di backup integrata (solo amministratori), oppure copia l'intera cartella a app chiusa.

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

## Stack tecnologico

[Rust](https://www.rust-lang.org) · [eframe/egui](https://github.com/emilk/egui) · [rusqlite](https://github.com/rusqlite/rusqlite) · [argon2](https://crates.io/crates/argon2) · [chrono](https://crates.io/crates/chrono) · [rfd](https://github.com/PolyMeilex/rfd)

## Contribuire

Contributi, segnalazioni di bug e idee sono benvenuti!

1. Fai un fork del repository
2. Crea un branch (`git checkout -b feature/nuova-funzione`)
3. Esegui `cargo fmt`, `cargo clippy` e `cargo test`
4. Apri una Pull Request

Per aggiungere una lingua, parti dai file in `src/i18n/`.

## Licenza

Distribuito sotto licenza **MIT**. Vedi il file `LICENSE` per i dettagli.
