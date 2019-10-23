//! The `SubotaiHash` is the element around which the Subotai network organizes itself.
//! It's 160 bits long, and it serves as an ID number to identify nodes, as a key to
//! identify storage entries, and optionally as a stored value.
//!
//! This module exposes utilities to create and inspect `SubotaiHash` structures. A
//! useful method is `sha1`, which allows you to create a sha-1 hash from some data,
//! which can then be used as a key for a storage entry.
use rand::{thread_rng, Rng};
use itertools;
use serde::{Serialize, Deserialize};
use std::ops::BitXor;
use std::fmt;
use std::fmt::Write;
use std::cmp::{PartialOrd, Ordering};
use sha1;

pub const HASH_SIZE : usize = 160;
pub const HASH_SIZE_BYTES : usize = HASH_SIZE / 8;

/// Subotai hash, a light wrapper over a li  ttle endian `HASH_SIZE` bit hash.
/// It can be generated randomly or via sha-1 of a given string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubotaiHash {
   pub raw : [u8; HASH_SIZE_BYTES],
}

impl SubotaiHash {
   /// Generates a blank hash (every bit set to 0).
   pub fn blank() -> SubotaiHash {
      SubotaiHash { raw : [0; HASH_SIZE_BYTES] }
   }

   /// Generates a random hash via kernel supplied entropy.
   pub fn random() -> SubotaiHash {
      let mut hash = SubotaiHash::blank();
//      thread_rng().fill_bytes(&mut hash.raw);
      hash
   }

   /// Generates a SHA-1 hash from a string.
   pub fn sha1(data: &str) -> SubotaiHash {
      let mut m = sha1::Sha1::new();
      m.reset();
      m.update(data.as_bytes());
      SubotaiHash {
         raw: m.digest().bytes(),
      }
   }

   /// Creates a random hash at a given XOR distance from another (height of their XOR value).
   pub fn random_at_distance(reference: &SubotaiHash, distance: usize) -> SubotaiHash {
      let mut random_hash = SubotaiHash::random();
      let distance_ones = (&random_hash ^ reference).into_ones();
      for index in distance_ones.rev() {
         random_hash.flip_bit(index);
         if let Some(height) = (&random_hash ^ reference).height() {
            if height == distance {
               return random_hash;
            } else if height < distance {
               random_hash.flip_bit(distance);
               return random_hash;
            }
         }
      }
       
      random_hash
   }

   /// Provides an iterator through the indices
   /// of each of its "0" bits.
   pub fn zeroes(&self) -> Zeroes {
      Zeroes {
         hash  : self,
         index : 0,
         rev   : HASH_SIZE
      }
   }

   /// Provides an iterator through the indices
   /// of each of its "1" bits.
   pub fn ones(&self) -> Ones {
      Ones {
         hash  : self,
         index : 0,
         rev   : HASH_SIZE
      }
   }

   /// Provides a consuming iterator through the 
   /// indices of each of its "0" bits.
   pub fn into_zeroes(self) -> IntoZeroes {
      IntoZeroes {
         hash  : self,
         index : 0,
         rev   : HASH_SIZE
      }
   }

   /// Provides a consuming iterator through the 
   /// indices of each of its "1" bits.
   pub fn into_ones(self) -> IntoOnes {
      IntoOnes {
         hash  : self,
         index : 0,
         rev   : HASH_SIZE
      }
   }

   /// Computes the bit index of the highest "1". Returns None for a blank hash.
   pub fn height(&self) -> Option<usize> {
      let last_nonzero_byte = self.raw.iter().enumerate().rev().find(|&pair| *pair.1 != 0);
      if let Some((index, byte)) = last_nonzero_byte {
         for bit in (0..8).rev() {
            if (byte & (1 << bit)) != 0 {
               return Some((8 * index + bit) as usize)
            }
         }
      }
      None
   }

   /// Flips a bit in the hash.
   pub fn flip_bit(&mut self, position : usize) {
      if position >= HASH_SIZE { return; }
      let byte = &mut self.raw[position / 8];
      *byte ^= 1 << (position % 8);
   }
}

impl fmt::Display for SubotaiHash {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      let mut leftpad_over = false;
      let mut hex = String::new();
      hex.push_str("0x[");
      for byte in self.raw.iter().rev() {
         if *byte > 0u8 {
            leftpad_over = true;
         }

         if leftpad_over {
            write!(&mut hex, "{:01$X}", byte, 2).unwrap();
         }
      }
      hex.push_str("]");
      write!(f, "{}", hex)
   }
}

/// Iterator through the indices of each '0' in a hash.
pub struct Zeroes<'a> { 
   hash  : &'a SubotaiHash,
   index : usize,
   rev   : usize
}

/// Iterator through the indices of each '1' in a hash.
pub struct Ones<'a> { 
   hash  : &'a SubotaiHash,
   index : usize,
   rev   : usize,
}

/// Consuming iterator through the indices of each '0' in a hash.
pub struct IntoZeroes { 
   hash  : SubotaiHash,
   index : usize,
   rev   : usize
}

/// Consuming iterator through the indices of each '1' in a hash.
pub struct IntoOnes { 
   hash  : SubotaiHash,
   index : usize,
   rev   : usize,
}

impl<'a> Iterator for Zeroes<'a> {
   type Item = usize;

   fn next(&mut self) -> Option<usize> {
      while self.index < self.rev {
         let value_at_index = self.hash.raw[self.index / 8] & (1 << (self.index % 8));
         self.index += 1;
         if value_at_index == 0 {
            return Some(self.index - 1);
         }
      }
      None
   }
}

impl<'a> Iterator for Ones<'a> {
   type Item = usize;

   fn next(&mut self) -> Option<usize> {
      while self.index < self.rev {
         let value_at_index = self.hash.raw[self.index / 8] & (1 << (self.index % 8));
         self.index += 1;
         if value_at_index != 0 {
            return Some(self.index - 1);
         }
      }
      None
   }
}

impl<'a> DoubleEndedIterator for Zeroes<'a> {
   fn next_back(&mut self) -> Option<usize> {
      while self.index < self.rev {
         let value_at_rev = self.hash.raw[(self.rev-1) / 8] & (1 << ((self.rev-1) % 8));
         self.rev -= 1;
         if value_at_rev == 0 {
            return Some(self.rev);
         }
      }
      None
   }
}

impl<'a> DoubleEndedIterator for Ones<'a> {
   fn next_back(&mut self) -> Option<usize> {
      while self.index < self.rev {
         let value_at_rev = self.hash.raw[(self.rev-1) / 8] & (1 << ((self.rev-1) % 8));
         self.rev -= 1;
         if value_at_rev != 0 {
            return Some(self.rev);
         }
      }
      None
   }
}

impl Iterator for IntoZeroes {
   type Item = usize;

   fn next(&mut self) -> Option<usize> {
      while self.index < self.rev {
         let value_at_index = self.hash.raw[self.index / 8] & (1 << (self.index % 8));
         self.index += 1;
         if value_at_index == 0 {
            return Some(self.index - 1);
         }
      }
      None
   }
}

impl Iterator for IntoOnes {
   type Item = usize;

   fn next(&mut self) -> Option<usize> {
      while self.index < self.rev {
         let value_at_index = self.hash.raw[self.index / 8] & (1 << (self.index % 8));
         self.index += 1;
         if value_at_index != 0 {
            return Some(self.index - 1);
         }
      }
      None
   }
}

impl DoubleEndedIterator for IntoZeroes {
   fn next_back(&mut self) -> Option<usize> {
      while self.index < self.rev {
         let value_at_rev = self.hash.raw[(self.rev-1) / 8] & (1 << ((self.rev-1) % 8));
         self.rev -= 1;
         if value_at_rev == 0 {
            return Some(self.rev);
         }
      }
      None
   }
}

impl DoubleEndedIterator for IntoOnes {
   fn next_back(&mut self) -> Option<usize> {
      while self.index < self.rev {
         let value_at_rev = self.hash.raw[(self.rev-1) / 8] & (1 << ((self.rev-1) % 8));
         self.rev -= 1;
         if value_at_rev != 0 {
            return Some(self.rev);
         }
      }
      None
   }
}

impl PartialOrd for SubotaiHash {
   fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      for (a,b) in self.raw.iter().rev().zip(other.raw.iter().rev()) {
         match a.cmp(b) {
            Ordering::Less => return Some(Ordering::Less),
            Ordering::Greater => return Some(Ordering::Greater),
            Ordering::Equal => ()
         }
      }
      None 
   }
}

impl Ord for SubotaiHash {
   fn cmp(&self, other: &Self) -> Ordering {
      match self.partial_cmp(other) {
         Some(order) => order,
         None => Ordering::Equal
      }
   }
}

impl<'a, 'b> BitXor<&'b SubotaiHash> for &'a SubotaiHash {
   type Output = SubotaiHash;

   fn bitxor (self, rhs: &'b SubotaiHash) -> SubotaiHash {
      let mut result = SubotaiHash::blank();
      for (d, a, b) in itertools::multizip((&mut result.raw, &self.raw, &rhs.raw)) {
         *d = a^b;
      }
      result
   }
}

impl BitXor for SubotaiHash {
   type Output = SubotaiHash;

   fn bitxor (mut self, rhs: Self) -> SubotaiHash {
      for (a, b) in self.raw.iter_mut().zip(rhs.raw.iter()) {
         *a ^= *b;
      }
      self
   }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_generation() {
       assert!(SubotaiHash::random() != SubotaiHash::random());
    }

    #[test]
    fn xor() {
       let alpha = SubotaiHash::random();
       let beta = SubotaiHash {
          raw: alpha.raw,
       };

       let reference_xor = &alpha ^ &beta;
       let value_xor = alpha ^ beta;

       for (a, b) in reference_xor.raw.iter().zip(value_xor.raw.iter()) {
          assert_eq!(*a, 0x00);
          assert_eq!(*b, 0x00);
       }
    }

    #[test]
    fn computing_height() {
       let mut test_hash = SubotaiHash::blank();
       assert!(test_hash.height().is_none());
       
       // First bit
       test_hash.raw[0] = 1;
       assert_eq!(test_hash.height(), Some(0));

       // Fourth bit (index 3)
       test_hash.raw[0] = test_hash.raw[0] | (1 << 3);
       assert_eq!(test_hash.height(), Some(3));

       // Last bit (index 159)
       test_hash.raw[19] = 1 << 7;
       assert_eq!(test_hash.height(), Some(159));
    }

    #[test]
    fn bit_flipping() {
       let mut test_hash = SubotaiHash::blank();
       test_hash.flip_bit(9);
       assert_eq!(test_hash.raw[1], 2);
       test_hash.flip_bit(9);
       assert_eq!(test_hash.raw[1], 0);
    }

    #[test]
    fn iterating_over_ones() {
       let mut test_hash = SubotaiHash::blank();
       let bits = vec![5usize,20,40];

       for bit in &bits {
          test_hash.flip_bit(*bit);
       }

       for (actual, expected) in test_hash.ones().zip(bits) {
          assert_eq!(actual, expected);
       }
    }

   #[test]
   fn random_at_a_distance() {
      let test_hash = SubotaiHash::random();
      let distance = 30usize;
      let new_hash = SubotaiHash::random_at_distance(&test_hash, distance);
      let distance_hash = test_hash ^ new_hash;
      assert_eq!(distance, (distance_hash).height().unwrap());
   }
}
