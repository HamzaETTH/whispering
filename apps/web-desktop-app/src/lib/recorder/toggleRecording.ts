import { apiKey } from '$lib/stores/apiKey';
import { audioSrc, outputText, recordingState } from '$lib/stores/recordingState';
import { pasteTextFromClipboard, writeTextToClipboard } from '$lib/system-apis/clipboard';
import { setAlwaysOnTop } from '$lib/system-apis/window';
import PleaseEnterAPIKeyToast from '$lib/toasts/PleaseEnterAPIKeyToast.svelte';
import SomethingWentWrongToast from '$lib/toasts/SomethingWentWrongToast.svelte';
import { transcribeAudioWithWhisperApi } from '$lib/transcribeAudioWithWhisperApi';
import toast from 'svelte-french-toast';
import { get } from 'svelte/store';
import { startRecording, stopRecording } from './mediaRecorder';
import { invoke } from '@tauri-apps/api/tauri';

export async function toggleRecording() {
	const apiKeyValue = get(apiKey);
	if (!apiKeyValue) {
		toast.error(PleaseEnterAPIKeyToast);
		return;
	}

	const recordingStateValue = get(recordingState);
	if (recordingStateValue === 'idle') {
		await setAlwaysOnTop(true);
		await startRecording();
		recordingState.set('recording');
		(async () => {
			while (get(recordingState) === 'recording') {
				await invoke('set_tray_icon', { state: 'recording' });
				await delay(500);
				await invoke('set_tray_icon', { state: 'idle' });
				await delay(500);
			}
		})();
	} else {
		try {
			const audioBlob = await stopRecording();
			audioSrc.set(URL.createObjectURL(audioBlob));
			recordingState.set('transcribing');
			await invoke('set_tray_icon', { state: 'transcribing' });
			await toast.promise(processRecording(audioBlob), {
				loading: 'Processing Whisper...',
				success: 'Copied to clipboard!',
				error: () => SomethingWentWrongToast
			});
		} catch (error) {
			console.error('Error occurred during transcription:', error);
		} finally {
			await setAlwaysOnTop(false);
			recordingState.set('idle');
			await invoke('set_tray_icon', { state: 'idle' });
		}
	}
}

function delay(ms) {
	return new Promise(resolve => setTimeout(resolve, ms));
}

async function processRecording(audioBlob: Blob) {
	const text = await transcribeAudioWithWhisperApi(audioBlob, get(apiKey));
	outputText.set(text);
	await writeTextToClipboard(text);
	await pasteTextFromClipboard();
}