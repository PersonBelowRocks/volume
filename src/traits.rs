use crate::types::*;
use num_traits::NumCast;
use num_traits::PrimInt;

pub trait VolumeIdx: Sized + Copy {
    /// Create a new index from X, Y, and Z components.
    /// # Panics
    /// Implementors may panic if `N` is not a valid type to build `Self` from.
    fn from_xyz<N: PrimInt>(x: N, y: N, z: N) -> Self;

    /// Cast this index to an array of an integer type.
    /// Returns `None` if the cast failed.
    fn array<T: NumCast + PrimInt>(self) -> Option<[T; 3]>;
}

/// Defines behaviour for accesses to a [`Volume`] for a given `Idx` type.
/// Think of this trait like [`std::ops::Index`] but for volumes.
/// This trait allows you to have a lot of fun with types in your volume.
/// One great thing you can do due to this trait is have types or an enum that represent vector spaces,
/// and you can index into the volume from a vector space of your choosing, and your implementation will
/// do the conversions/transformations for you.
///
/// # Example
///
/// Let's look at an example you might see in a voxel game.
/// ```rust
/// // Prelude contains this trait + all other stuff we need.
/// use volume::prelude::*;
///
/// type ChunkStorage = [[[u32; 10]; 10]; 10];
/// type VoxelPos = [i64; 3]; // Represents a 3D position of a voxel.
///
/// // In our game we'll divide world/terrain data into chunks that can be generated, saved, and loaded
/// // individually to split up the work. Just like in Minecraft. Each chunk has a position in the world,
/// // and we can talk about voxels in terms of their position in the world OR their position in their chunk!
/// // Their position in their world is their "worldspace position", and their position in the chunk is their "localspace position".
/// // For example, say you have a chunk at position [30, 30, 30], and there's a voxel at position [5, 6, 7] inside that chunk.
/// // That voxel has the worldspace position of [30, 30, 30] + [5, 6, 7] = [30 + 5, 30 + 6, 30 + 7] = [35, 36, 37].
/// struct Chunk {
///     pos: VoxelPos,
///     storage: ChunkStorage,
/// }
///
/// impl Chunk {
///     // Convenience function to make a new chunk.
///     fn new(item: u32, pos: VoxelPos) -> Self {
///         Self {
///             pos,
///             storage: [[[item; 10]; 10]; 10]
///         }
///     }
/// }
///
/// // Here's where stuff gets fun, this type represents vector spaces of an index into a chunk.
/// // Recall what we mentioned above, that voxels have a position in the world, and in their chunk.
/// enum ChunkIndex {
///     // Represents a voxel's position in the world.
///     Worldspace(VoxelPos),
///     // Represents a voxel's position in its chunk.
///     Localspace(VoxelPos),
/// }
///
/// // Now let's implement this behaviour!
/// // We'll return None if the given index isn't inside this chunk aka. out of bounds.
/// // In our previously mentioned chunk at position [30, 30, 30], a position of [35, 35, 35] in worldspace is a valid index,
/// // but the same position in localspace is NOT a valid index! If we convert [35, 35, 35] to localspace we'd get [5, 5, 5],
/// // which is a valid localspace index but NOT a valid worldspace index! [5, 5, 5] in localspace and [35, 35, 35] in worldspace both
/// // refer to the same voxel.
/// impl VolumeAccess<ChunkIndex> for Chunk {
///     fn access(this: &Self, idx: ChunkIndex) -> Option<&Self::Item> {
///         match idx {
///             ChunkIndex::Worldspace(pos) => {
///                 // In case it's a little too cluttered and not obvious, we're subtracting vectors here.
///                 // You'd probably want to use some math crate for this in an actual scenario.
///                 let [mx, my, mz] = this.bounding_box().min();
///                 let ls_idx = [
///                     pos[0] - mx,
///                     pos[1] - my,
///                     pos[2] - mz,
///                 ];
///                 
///                 // Bit of recursion here! The optimizer is probably (hopefully) going to just inline everything so this is fine.
///                 Self::access(this, ChunkIndex::Localspace(ls_idx))
///             },
///             ChunkIndex::Localspace(pos) => {
///                 let [x, y, z] = pos;
///                 this.storage.get(x as usize)?.get(y as usize)?.get(z as usize)
///             }
///         }
///     }
/// }
///
/// // This is the exact same as above but with get_mut() instead of get()!
/// impl VolumeMutAccess<ChunkIndex> for Chunk {
///     fn access_mut(this: &mut Self, idx: ChunkIndex) -> Option<&mut Self::Item> {
///         match idx {
///             ChunkIndex::Worldspace(pos) => {
///                 let [mx, my, mz] = this.bounding_box().min();
///                 let ls_idx = [
///                     pos[0] - mx,
///                     pos[1] - my,
///                     pos[2] - mz,
///                 ];
///
///                 Self::access_mut(this, ChunkIndex::Localspace(ls_idx))
///             },
///             ChunkIndex::Localspace(pos) => {
///                 let [x, y, z] = pos;
///                 this.storage.get_mut(x as usize)?.get_mut(y as usize)?.get_mut(z as usize)
///             }
///         }
///     }
/// }
///
/// // The Volume trait has a bunch of methods on it with default implementations like swap(). These methods can require you to implement
/// // VolumeAccess<T> because they use "special" indices of type T.
/// impl Volume for Chunk {
///     type Item = u32;
///     
///     fn bounding_box(&self) -> BoundingBox {
///         let pos1 = self.pos;
///         let pos2 = [
///             pos1[0] + 10,
///             pos1[1] + 10,
///             pos1[2] + 10,
///         ];
///         
///         BoundingBox::new(pos1, pos2)
///     }
/// }
///
/// let mut chunk = Chunk::new(42, [10, 10, 10]);
///
/// // Use the provided Volume::get methods instead of VolumeAccess::access to interact with the volume.
/// // Volume::get sort of just wraps VolumeAccess::access and is a lot more readable.
/// assert_eq!(chunk.get(ChunkIndex::Worldspace([15, 14, 17])), Some(&42));
/// assert_eq!(chunk.get(ChunkIndex::Localspace([15, 14, 17])), None); // Invalid localspace index!
///
/// // Let's swap this voxel now.
/// assert_eq!(chunk.swap(ChunkIndex::Worldspace([15, 14, 17]), 21), Some(42));
///
/// // Both of these positions refer to the same voxel, just in different vector spaces.
/// assert_eq!(chunk.get(ChunkIndex::Worldspace([15, 14, 17])), Some(&21));
/// assert_eq!(chunk.get(ChunkIndex::Localspace([5, 4, 7])), Some(&21));
/// ```
pub trait VolumeAccess<Idx>: Volume {
    fn access(this: &Self, idx: Idx) -> Option<&Self::Item>;
}

/// Like [`VolumeAccess`] but provides a mutable borrow instead.
/// If it doesn't make sense for your volume to return a mutable reference to its items
/// (for example if your volume performs some transformation in its accessors, and stores a different type than what it returns)
/// then you should implement [`VolumeSwapper`] to mutate the volume instead.
pub trait VolumeMutAccess<Idx>: Volume {
    fn access_mut(this: &mut Self, idx: Idx) -> Option<&mut Self::Item>;
}

/// Swaps the value at the given `idx` with the provided `item`, and returns the old value if it existed (otherwise returns [`None`]).
pub trait VolumeSwapper<Idx>: Volume {
    fn swap(this: &mut Self, idx: Idx, item: Self::Item) -> Option<Self::Item>;
}

/// Provides a bunch of convenience methods related to volumes. Should be implemented for any type
/// acting as a volume. Generic functions operating on volumes should have this trait as a bound for their types.
pub trait Volume: Sized {
    /// The item that this volume will expose in its API. Should correspond with whatever is stored in the underlying memory of the volume.
    type Item;

    /// Wrapper around [`VolumeAccess<Idx>::access`], and requires [`VolumeAccess<Idx>`] to be implemented for the volume.
    /// Returns [`None`] if the given `idx` is invalid (depends on the implementation of [`VolumeAccess<Idx>`]).
    #[inline]
    fn get<Idx>(&self, idx: Idx) -> Option<&Self::Item>
    where
        Self: VolumeAccess<Idx>,
    {
        <Self as VolumeAccess<Idx>>::access(self, idx)
    }

    /// Wrapper around [`VolumeMutAccess<Idx>::access_mut`], and requires [`VolumeMutAccess<Idx>`] to be implemented for the volume.
    /// Returns [`None`] if the given `idx` is invalid (depends on the implementation of [`VolumeMutAccess<Idx>`]).
    #[inline]
    fn get_mut<Idx>(&mut self, idx: Idx) -> Option<&mut Self::Item>
    where
        Self: VolumeMutAccess<Idx>,
    {
        <Self as VolumeMutAccess<Idx>>::access_mut(self, idx)
    }

    /// Wrapper around [`VolumeSwapper<Idx>::swap`], and requires [`VolumeSwapper<Idx>`] to be implemented for the volume.
    /// Returns [`None`] if the given `idx` is invalid (depends on the implementation of [`VolumeSwapper<Idx>`]).
    #[inline]
    fn swap<Idx>(&mut self, idx: Idx, item: Self::Item) -> Option<Self::Item>
    where
        Self: VolumeSwapper<Idx>,
    {
        <Self as VolumeSwapper<Idx>>::swap(self, idx, item)
    }

    /// Get a [`BoundingBox`] representing this volume's bounds. Implementors must assume that any position within the bounding box is a valid worldspace index
    /// so that [`Volume::get`] and [`Volume::get_mut`] do not return [`None`] when given the index.
    ///
    /// Much like an iterator's size hint, unsafe code SHOULD NOT rely on [`Volume::bounding_box`] for anything potentially bad.
    ///
    /// [`BoundingBox`]es may change in the future to allow for different vector spaces.
    fn bounding_box(&self) -> BoundingBox;

    /// Checks if this volume contains the worldspace index.
    #[inline]
    fn contains<Idx: VolumeIdx>(&self, idx: Idx) -> bool {
        self.bounding_box().contains::<Idx>(idx)
    }

    /// Iterate over the positions of this volume's bounding box.
    /// Be mindful of the vector space these indices belong to if you use them to index into the volume!
    /// This is likely going to change in the future, and you'll be able to choose the vector space to iterate in.
    #[inline]
    fn iter_indices(&self) -> BoundingBoxIterator {
        self.bounding_box().into_iter()
    }

    /// Iterate over the elements in this volume.
    /// Requires [`VolumeAccess<[i64; 3]>`] to be implemented for the volume.
    /// This may change in the future if iteration over indices in different vector spaces is added.
    #[inline]
    fn iter(&self) -> VolumeIterator<'_, Self>
    where
        Self: VolumeAccess<[i64; 3]>,
    {
        VolumeIterator {
            volume: self,
            bb_iterator: self.iter_indices(),
        }
    }
}
