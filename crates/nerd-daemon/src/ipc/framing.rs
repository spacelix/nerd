use std::io;

#[cfg(test)]
use nerd_core::codec::decode_payload;
use nerd_core::codec::{FRAME_PREFIX_BYTES, MAX_FRAME_BYTES, encode_frame};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::{Duration, timeout};

#[cfg(test)]
pub(super) async fn read_message<R, T>(reader: &mut R) -> io::Result<Option<T>>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let mut prefix = [0_u8; FRAME_PREFIX_BYTES];
    let first = reader.read(&mut prefix[..1]).await?;
    if first == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut prefix[1..]).await?;

    let length = u32::from_le_bytes(prefix) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("IPC frame exceeds {MAX_FRAME_BYTES} bytes"),
        ));
    }

    let mut payload = vec![0_u8; length];
    reader.read_exact(&mut payload).await?;
    decode_payload(&payload)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
pub(super) async fn read_message_bounded<R, T>(
    reader: &mut R,
    idle_timeout: Duration,
    completion_timeout: Duration,
) -> io::Result<Option<T>>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let mut prefix = [0_u8; FRAME_PREFIX_BYTES];
    let first = timeout(idle_timeout, reader.read(&mut prefix[..1]))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IPC request idle timeout"))??;
    if first == 0 {
        return Ok(None);
    }

    timeout(completion_timeout, async {
        reader.read_exact(&mut prefix[1..]).await?;
        let length = u32::from_le_bytes(prefix) as usize;
        if length > MAX_FRAME_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("IPC frame exceeds {MAX_FRAME_BYTES} bytes"),
            ));
        }
        let mut payload = vec![0_u8; length];
        reader.read_exact(&mut payload).await?;
        decode_payload(&payload)
            .map(Some)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IPC frame completion timeout"))?
}

pub(super) async fn read_payload_bounded<R>(
    reader: &mut R,
    idle_timeout: Duration,
    completion_timeout: Duration,
) -> io::Result<Option<Vec<u8>>>
where
    R: AsyncRead + Unpin,
{
    let mut prefix = [0_u8; FRAME_PREFIX_BYTES];
    let first = timeout(idle_timeout, reader.read(&mut prefix[..1]))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IPC request idle timeout"))??;
    if first == 0 {
        return Ok(None);
    }

    timeout(completion_timeout, async {
        reader.read_exact(&mut prefix[1..]).await?;
        let length = u32::from_le_bytes(prefix) as usize;
        if length > MAX_FRAME_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("IPC frame exceeds {MAX_FRAME_BYTES} bytes"),
            ));
        }
        let mut payload = vec![0_u8; length];
        reader.read_exact(&mut payload).await?;
        Ok(Some(payload))
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IPC frame completion timeout"))?
}

pub(super) async fn write_message<W, T>(writer: &mut W, message: &T) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
    T: DeserializeOwned + Serialize,
{
    let frame =
        encode_frame(message).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    writer.write_all(&frame).await?;
    writer.flush().await
}

#[cfg(test)]
mod tests {
    use std::{io, time::Duration};

    use nerd_core::ipc::RequestEnvelope;
    use tokio::io::{AsyncWriteExt, duplex};

    use super::read_message_bounded;

    #[test]
    fn idle_and_partial_frames_time_out() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");
        runtime.block_on(async {
            let (_writer, mut reader) = duplex(64);
            let idle = read_message_bounded::<_, RequestEnvelope>(
                &mut reader,
                Duration::from_millis(10),
                Duration::from_secs(1),
            )
            .await
            .expect_err("idle frame must time out");
            assert_eq!(idle.kind(), io::ErrorKind::TimedOut);

            let (mut writer, mut reader) = duplex(64);
            writer.write_all(&[1]).await.expect("write partial prefix");
            let partial = read_message_bounded::<_, RequestEnvelope>(
                &mut reader,
                Duration::from_secs(1),
                Duration::from_millis(10),
            )
            .await
            .expect_err("partial frame must time out");
            assert_eq!(partial.kind(), io::ErrorKind::TimedOut);
        });
    }
}
