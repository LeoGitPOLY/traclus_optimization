# rust_impl

## Downloading the Application

Go to the [**Releases page**](https://github.com/LeoGitPOLY/traclus_optimization/releases/v1.0.0) of this repository.
Under **Assets**, download the file for your platform:

| Platform                       | File to download          |
| ------------------------------ | ------------------------- |
| Windows (64-bit)               | `rust_impl.exe`           |
| macOS Intel                    | `rust_impl-mac-intel.zip` |
| macOS Apple Silicon (M1/M2/M3) | `rust_impl-mac-arm.zip`   |

---

## Running the Application

### Windows

1. Download `rust_impl.exe`
2. Double-click it to launch

> If Windows shows a "Windows protected your PC" SmartScreen warning, click **More info** → **Run anyway**.

---

### macOS

1. Download the `.zip` for your Mac and double-click it to unzip → you get `rust_impl.app`
2. **Do not double-click it yet.** Right-click (or Control+click) the app → **Open**
3. A dialog will appear — click **Open Anyway**
4. From now on you can double-click it normally

> If you see **"rust_impl is damaged and can't be opened"**:
>
> 1. Open **Terminal** (press ⌘+Space, type `Terminal`, press Enter)
> 2. Type `xattr -cr ` (with a space at the end)
> 3. Drag and drop `rust_impl.app` into the Terminal window
> 4. Press Enter, then try opening the app again

> **Not sure which Mac you have?** Click the Apple menu () → **About This Mac**.
>
> - If it says **Apple M1 / M2 / M3** → download `rust_impl-mac-arm.zip`
> - If it says **Intel** → download `rust_impl-mac-intel.zip`

---

## Data

Sample input files are provided in the [`/data`](./data) folder of this repository:

| File                     | Format                           |
| ------------------------ | -------------------------------- |
| `sample_with_header.txt` | Tab-separated, with a header row |
| `sample_no_header.txt`   | Tab-separated, no header row     |

### Accepted file format

The application accepts **tab-separated** (`.txt`) or **comma-separated** (`.csv`) files.  
Each data row must contain either **5 or 6 fields**:

| Field     | Type    | Description                           |
| --------- | ------- | ------------------------------------- |
| `name`    | text    | _(optional)_ Label for the line       |
| `weight`  | integer | Number of trips on this OD line       |
| `x_start` | decimal | X coordinate of the origin point      |
| `y_start` | decimal | Y coordinate of the origin point      |
| `x_end`   | decimal | X coordinate of the destination point |
| `y_end`   | decimal | Y coordinate of the destination point |

**With name (6 fields):**

```
route_A	3	48.8566	2.3522	48.8606	2.3376
```

**Without name (5 fields):**

```
3	48.8566	2.3522	48.8606	2.3376
```

> A header row is automatically detected and skipped if the first line contains non-numeric values.  
> Empty lines are ignored.

## Feedback & Bug Reports

Found a bug or have a suggestion for improvement?  
Send an email to [leonard.pouliot@etud.polymtl.ca](mailto:leonard.pouliot@etud.polymtl.ca) with:

- A short description of the bug or idea
- Your platform (Windows / macOS Intel / macOS ARM)
- The input file used _(if relevant)_
