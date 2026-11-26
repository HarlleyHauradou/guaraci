# 2. Units and dimensional analysis

Crate `guaraci-units`.

## Dimensions

Every value is stored in SI units together with its dimension, a vector of
integer exponents `[L, M, T, A]`: length, mass, time and plane angle.

| Quantity | Dimension |
|---|---|
| energy | M L² T⁻² |
| activity | T⁻¹ |
| absorbed dose | L² T⁻² (J/kg) |
| density | M L⁻³ |

The angle is a dimension of its own (in SI it is dimensionless), so that
`radius = 30 deg` is rejected.

## Unit table

The table (`table.rs`) lists **only** units in common use in radiation
transport. An unusual symbol (such as the megametre) is more likely a typo
than an intention, so it is not accepted. Exact factors:
1 eV = 1.602176634×10⁻¹⁹ J (SI 2019) and 1 Ci = 3.7×10¹⁰ Bq; the year is the
Julian year (365.25 d).

## Resolving a symbol

1. The exact spelling comes first.
2. Without an exact match, symbols equal up to letter case are searched. The
   symbol is accepted if there is a single candidate, or if all candidates
   have the same factor and dimension (`l`/`L`).
3. If candidates have different meanings, it is an error, and the message
   lists the options.

A test guarantees that the current table has no pair of symbols that differ
only by case and have different meanings.

## Compound units

An expression is a product of atoms separated by `*`, `.` or `·`, or a
quotient separated by `/`. Each atom may have an integer exponent (`cm3`,
`cm^3`, `m^-3`). Each `/` inverts only the next atom, so `a/b/c` = a·b⁻¹·c⁻¹.

## Values without a unit

Each input key declares the quantity it expects (`Kind`). A number without a
unit gets the default unit of that quantity, and every occurrence is recorded
(`implicit_units`) so that it appears in `check` and in the report.
