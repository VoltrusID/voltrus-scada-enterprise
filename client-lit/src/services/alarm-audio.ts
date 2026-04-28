/**
 * Alarm audio manager.
 * No React hooks, just a plain class.
 */
export class AlarmAudio {
  private _audio: HTMLAudioElement | null = null;
  private _muted = false;
  private _playing = false;

  constructor() {
    if (typeof Audio !== 'undefined') {
      this._audio = new Audio('/alarm.wav');
      this._audio.loop = true;
      this._audio.volume = 0.5;
    }
  }

  start() {
    if (!this._audio || this._muted || this._playing) return;
    this._audio.play().catch(() => {});
    this._playing = true;
  }

  stop() {
    if (!this._audio) return;
    this._audio.pause();
    this._audio.currentTime = 0;
    this._playing = false;
  }

  toggleMute() {
    this._muted = !this._muted;
    if (this._muted) this.stop();
    return this._muted;
  }

  get isMuted() { return this._muted; }
  get isPlaying() { return this._playing; }
}

export const alarmAudio = new AlarmAudio();
