use std::fmt::Debug;

use rtrb::CopyToUninit;
use rtrb::Producer;

pub fn push_partial_slice<T>(queue: &mut Producer<T>, slice: &[T]) -> usize
where
    T: Copy + Debug,
{
    use rtrb::chunks::ChunkError::TooFewSlots;

    let slice_len = slice.len();

    let mut chunk = match queue.write_chunk_uninit(slice_len) {
        Ok(chunk) => chunk,
        // Remaining slots are returned, this will always succeed:
        Err(TooFewSlots(n)) => queue.write_chunk_uninit(n).unwrap(),
    };

    let end = chunk.len();
    let (first, second) = chunk.as_mut_slices();
    let mid = first.len();
    slice[..mid].copy_to_uninit(first);
    slice[mid..end].copy_to_uninit(second);

    // SAFETY: All slots have been initialized
    unsafe {
        chunk.commit_all();
    }

    slice_len - end
}
