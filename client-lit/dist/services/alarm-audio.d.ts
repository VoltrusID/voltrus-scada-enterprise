/**
 * Alarm audio manager.
 * No React hooks, just a plain class.
 */
export declare class AlarmAudio {
    private _audio;
    private _muted;
    private _playing;
    constructor();
    start(): void;
    stop(): void;
    toggleMute(): boolean;
    get isMuted(): boolean;
    get isPlaying(): boolean;
}
export declare const alarmAudio: AlarmAudio;
//# sourceMappingURL=alarm-audio.d.ts.map