use crate::errors::SonioxLiveErrors;
use crate::soniox::session::{SonioxSessionReader, SonioxSessionWriter};
use crate::soniox::WsStream;
use crate::types::soniox::SonioxTranscriptionRequest;
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use tungstenite::client::IntoClientRequest;
use tungstenite::Utf8Bytes;

pub struct SonioxConnection {
    ws_stream: WsStream,
}

impl SonioxConnection {
    pub async fn connect(url: impl IntoClientRequest) -> Result<Self, SonioxLiveErrors> {
        let request = url.into_client_request()?;
        let (ws_stream, _) = connect_async(request).await?;

        Ok(Self { ws_stream })
    }

    pub async fn into_session(
        self,
        request: &SonioxTranscriptionRequest,
    ) -> Result<(SonioxSessionWriter, SonioxSessionReader), SonioxLiveErrors> {
        let (w, r) = self.ws_stream.split();
        let mut writer = SonioxSessionWriter(w);
        let reader = SonioxSessionReader(r);
        let bytes = serde_json::to_vec(request)?;
        writer.send_text(Utf8Bytes::try_from(bytes)?).await?;

        Ok((writer, reader))
    }
}
