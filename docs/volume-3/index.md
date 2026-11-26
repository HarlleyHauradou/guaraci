# Volume III — Graphical Interface

:::{note} In development
The graphical interface does not exist yet. This page describes what it will
do and how this volume will be organised. Everything the interface does can
also be done from the terminal, with the commands of
[Volume II](../volume-2/index.md).
:::

The interface is a desktop program written in Rust (egui), inspired by Flair
for FLUKA. It works **on the same `.gci` files**: nothing is hidden inside the
interface, and the input file remains the lab notebook of the simulation.

## Planned chapters

1. **Overview:** windows, projects and files.
2. **Editor:**
   - syntax highlighting of `.gci`;
   - checking while typing, with the same messages as `guaraci check`;
   - completion of material, volume and nuclide names.
3. **Geometry:**
   - 2D slices (axial, coronal, sagittal) coloured by material;
   - 3D view;
   - clicking a volume jumps to its section in the editor.
4. **Materials and nuclides:** browsers of the material database and of the
   decay data (lines, half-life, mean energies).
5. **Running:** start a simulation, follow its progress, read the log and stop it.
6. **Results:**
   - dose map over the geometry, with a colour scale;
   - dose profiles;
   - dose-volume histograms (DVH);
   - uncertainty map;
   - spectra.
7. **Export:** figures, tables and reports.
