use crate::Phylogeny;

const example_1: Phylogeny<&str, f32> = Phylogeny::from(&[("A", "B", 1.0)]);
        //     ("A", "B", 1),
        //     ("A", "C", 1),
        //     ("A", "D", 1),
        //     ("B", "D", 1),
        //     ("C", "F", 1),
        //     ("C", "G", 1),
        //     ("D", "E", 1),
        //     ("E", "G", 1),
        //     ("E", "H", 1),
        //     ("F", "G", 1),
        // ]
    
    // /// Returns the `Example 1` [Phylogeny], shown in the documentation [Introduction](#introduction).
    // ///
    // /// # Examples
    // ///
    // /// ```rust
    // /// use rebar_phylo::Phylogeny;
    // /// let phylo: Phylogeny<&str, usize> = Phylogeny::example_1();
    // /// ```
    // pub fn example_1() -> Phylogeny<&'static str, usize> {
    //     let data = vec![
    //         ("A", "B", 1),
    //         ("A", "C", 1),
    //         ("A", "D", 1),
    //         ("B", "D", 1),
    //         ("C", "F", 1),
    //         ("C", "G", 1),
    //         ("D", "E", 1),
    //         ("E", "G", 1),
    //         ("E", "H", 1),
    //         ("F", "G", 1),
    //     ];

    //     let phylo: Phylogeny<&str, usize> = Phylogeny::from(data);
    //     phylo
    // }