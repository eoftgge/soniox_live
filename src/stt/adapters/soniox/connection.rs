use super::session::{SonioxSessionReader, SonioxSessionWriter, WsStream};
use super::types::SonioxTranscriptionRequest;
use crate::errors::OmniSttErrors;
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use tungstenite::Utf8Bytes;
use tungstenite::client::IntoClientRequest;

pub struct SonioxConnection {
    ws_stream: WsStream,
}

impl SonioxConnection {
    pub async fn connect(url: impl IntoClientRequest) -> Result<Self, OmniSttErrors> {
        let request = url.into_client_request()?;
        let (ws_stream, _) = connect_async(request).await?;

        Ok(Self { ws_stream })
    }

    pub async fn into_session(
        self,
        request: &SonioxTranscriptionRequest,
    ) -> Result<(SonioxSessionWriter, SonioxSessionReader), OmniSttErrors> {
        let (w, r) = self.ws_stream.split();
        let mut writer = SonioxSessionWriter(w);
        let reader = SonioxSessionReader(r);
        let bytes = serde_json::to_vec(request)?;
        writer.send_text(Utf8Bytes::try_from(bytes)?).await?;

        Ok((writer, reader))
    }
}
