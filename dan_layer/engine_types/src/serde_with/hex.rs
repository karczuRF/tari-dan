//   Copyright 2022. The Tari Project
//
//   Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//   following conditions are met:
//
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//   disclaimer.
//
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
//   following disclaimer in the documentation and/or other materials provided with the distribution.
//
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
//   products derived from this software without specific prior written permission.
//
//   THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//   INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//   DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//   SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//   SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//   WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
//   USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S: Serializer, T: AsRef<[u8]>>(v: &T, s: S) -> Result<S::Ok, S::Error> {
    if s.is_human_readable() {
        let st = hex::encode(v.as_ref());
        s.serialize_str(&st)
    } else {
        s.serialize_bytes(v.as_ref())
    }
}

/// Use a serde deserializer to serialize the hex string of the given object.
pub fn deserialize<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: for<'a> TryFrom<&'a [u8]>,
{
    let bytes = if d.is_human_readable() {
        let hex = <String as Deserialize>::deserialize(d)?;
        hex::decode(&hex).map_err(serde::de::Error::custom)?
    } else {
        <Vec<u8> as Deserialize>::deserialize(d)?
    };

    let value = T::try_from(&bytes).map_err(|_| serde::de::Error::custom("Failed to convert bytes to T"))?;
    Ok(value)
}

pub mod vec {
    use serde::ser::SerializeSeq;

    use super::*;

    pub fn serialize<S: Serializer, T: AsRef<[u8]>>(v: &[T], s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            let mut seq = s.serialize_seq(Some(v.len()))?;
            for item in v {
                seq.serialize_element(&hex::encode(item.as_ref()))?;
            }
            seq.end()
        } else {
            let mut seq = s.serialize_seq(Some(v.len()))?;
            for item in v {
                seq.serialize_element(&item.as_ref())?;
            }
            seq.end()
        }
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: for<'a> TryFrom<&'a [u8]>,
    {
        let vec = if d.is_human_readable() {
            let strs = <Vec<String> as Deserialize>::deserialize(d)?;
            strs.iter()
                .map(|s| hex::decode(s).map_err(serde::de::Error::custom))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            <Vec<Vec<u8>> as Deserialize>::deserialize(d)?
        };

        let values = vec
            .into_iter()
            .map(|v| T::try_from(v.as_slice()).map_err(|_| serde::de::Error::custom("Failed to convert bytes to T")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(values)
    }
}

pub mod option {
    use super::*;

    pub fn serialize<S: Serializer, T: AsRef<[u8]>>(v: &Option<T>, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            match v {
                Some(v) => {
                    let st = hex::encode(v.as_ref());
                    s.serialize_some(&st)
                },
                None => s.serialize_none(),
            }
        } else {
            match v {
                Some(v) => s.serialize_some(v.as_ref()),
                None => s.serialize_none(),
            }
        }
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: for<'a> TryFrom<&'a [u8]>,
    {
        let bytes = if d.is_human_readable() {
            let hex = <Option<String> as Deserialize>::deserialize(d)?;
            hex.as_ref()
                .map(hex::decode)
                .transpose()
                .map_err(serde::de::Error::custom)?
        } else {
            <Option<Vec<u8>> as Deserialize>::deserialize(d)?
        };

        let value = bytes
            .as_ref()
            .map(|b| T::try_from(b.as_slice()))
            .transpose()
            .map_err(|_| serde::de::Error::custom("Failed to convert bytes to T"))?;
        Ok(value)
    }
}
