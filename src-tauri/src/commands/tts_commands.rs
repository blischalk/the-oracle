use tauri::State;

use crate::AppState;

/// Synthesize speech using the OpenAI TTS API and return raw MP3 bytes.
///
/// The API key is retrieved from the keychain so it never passes through the
/// frontend. The caller is responsible for playing the returned bytes.
#[tauri::command]
pub async fn synthesize_speech(
    text: String,
    voice: String,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let api_key = state
        .keychain_service
        .get_api_key("openai")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            "No OpenAI API key found. Add one in Settings to use OpenAI narration.".to_string()
        })?;

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/audio/speech")
        .bearer_auth(&api_key)
        .json(&serde_json::json!({
            "model": "tts-1-hd",
            "input": text,
            "voice": voice,
        }))
        .send()
        .await
        .map_err(|e| format!("TTS request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI TTS error {status}: {body}"));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read TTS response: {e}"))?;

    Ok(bytes.to_vec())
}
