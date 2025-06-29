// use lizstate_tiling::tileset::SubsetOf;
// use lizstate_tiling::tileset::TileSet;
// use lizstate_tiling::tiling::Tile;

// #[derive(Clone, Copy, PartialEq, Eq)]
// struct AsTile<T: 'static + Copy + Eq>(T);
// impl Tile for AsTile<i32> {}
// impl<T: 'static + Copy + Eq> From<T> for AsTile<T> {
//     fn from(value: T) -> Self {
//         Self(value)
//     }
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// struct Z;
// impl TileSet for Z {
//     type Element = AsTile<i32>;
//     fn contains(&self, _: &Self::Element) -> bool {
//         true
//     }
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// struct N;
// impl TileSet for N {
//     type Element = AsTile<i32>;
//     fn contains(&self, el: &Self::Element) -> bool {
//         el.0 >= 0
//     }
// }

// impl SubsetOf<Z> for N {
//     fn get_super(&self) -> Z {
//         Z
//     }
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// struct KN(i32);
// impl TileSet for KN {
//     type Element = AsTile<i32>;

//     fn contains(&self, el: &Self::Element) -> bool {
//         el.0 >= 0 && (el.0 % self.0) == 0
//     }
// }

// impl SubsetOf<N> for KN {
//     fn get_super(&self) -> N {
//         N
//     }
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// struct KNAndEven(KN);
// impl TileSet for KNAndEven {
//     type Element = AsTile<i32>;

//     fn contains(&self, el: &Self::Element) -> bool {
//         self.0.contains(el) && el.0 % 2 == 0
//     }
// }

// impl SubsetOf<KN> for KNAndEven {
//     fn get_super(&self) -> KN {
//         self.0
//     }
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// struct Empty<T: TileSet>(T);
// impl<T: TileSet> TileSet for Empty<T> {
//     type Element = T::Element;

//     fn contains(&self, _: &Self::Element) -> bool {
//         false
//     }
// }

// impl<T: TileSet> SubsetOf<T> for Empty<T> {
//     fn get_super(&self) -> T {
//         self.0
//     }
// }

// #[test]
// fn empty_is_subset_of_everything() {
//     let z = Z;
//     let kn_and_even = KNAndEven(KN(5));

//     let empty_z = Empty(z);
//     let empty_kn_and_even = Empty(kn_and_even);

//     let zero = z.try_prove(&AsTile(0)).unwrap();
//     assert!(empty_z.subset_try_prove(&zero).is_none());

//     let ten = kn_and_even.try_prove(&AsTile(10)).unwrap();
//     assert!(empty_kn_and_even.subset_try_prove(&ten).is_none())
// }
