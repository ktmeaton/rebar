# rebarg

A library for ancestral recombination graphs (ARG).

# Introduction

- The nodes (`N`) can be a wide variety of types (ex. [`str`], [`String`], [`usize`](core::primitive::str), [`Node`], etc.).
- The branches (`B`) must be a type that can be cast into an [`f32`] for the length.
- See the [Implementation](#impl-Phylogeny<N,+B>) section for the allowed types based on traits.
- See the [`Node`] and [`Branch`] structs for examples of complex data types.

```mermaid
graph TD;

  subgraph Legend
    direction LR;
    D1[ ] --->|Non-Recombination| D2[ ];
    style D1 height:0px;
    style D2 height:0px;
    D3[ ] -..->|Recombination| R1[ ];
    style D3 height:0px;
    style R1 height:0px;
  end

  subgraph Example 1
    direction LR;
    0["A"]:::default-->|1|1["B"]:::default;
    0["A"]:::default-->|1|2["C"]:::default;
    0["A"]:::default-.->|1|3["D"]:::recombinant;
    1["B"]:::default-.->|1|3["D"]:::recombinant;
    3["D"]:::recombinant-->|1|6["E"]:::default;
    6["E"]:::default-.->|1|5["G"]:::recombinant;
    6["E"]:::default-->|1|7["H"]:::default;
    2["C"]:::default-->|1|4["F"]:::default;
    2["C"]:::default-.->|1|5["G"]:::recombinant;
    4["F"]:::default-.->|1|5["G"]:::recombinant;
  end

classDef default stroke:#1f77b4
classDef recombinant stroke:#ff7f0e

linkStyle 1,4,5,7,10,11 stroke:#ff7f0e
linkStyle 0,2,3,6,8,9 stroke:#1f77b4
```