import { convertFileSrc, invoke } from '@tauri-apps/api/core';

export interface Song {
  id: number;
  folder_id: number;
  title: string;
  artist: string;
  album: string;
  genre: string;
  duration: number;
  file_path: string;
  file_size: number;
  cover_path?: string | null;
  format?: string;
  bitrate?: number;
  sample_rate?: number;
  bit_depth?: number;
  is_lossless?: boolean;
  waveform?: string | null;
  lyrics?: string | null;
  is_favorite?: boolean;
  replay_gain?: number | null;
}

export interface AiQueueItem {
  song: Song;
  matchScore: number;
  reason: string;
}

// Interface buat nyimpen histori lagu + konteks antreannya
export interface HistoryItem {
  song: Song;
  queue: Song[];
}

const getInitialVolume = (): number => {
  if (typeof window === 'undefined') return 0.8;
  const saved = localStorage.getItem('coeg_volume');
  return saved !== null ? parseFloat(saved) : 0.8;
};

const getInitialMute = (): boolean => {
  if (typeof window === 'undefined') return false;
  return localStorage.getItem('coeg_muted') === 'true';
};

const getInitialShuffle = (): boolean => {
  if (typeof window === 'undefined') return false;
  return localStorage.getItem('coeg_shuffle') === 'true';
};

const getInitialRepeat = (): 'off' | 'all' | 'one' => {
  if (typeof window === 'undefined') return 'off';
  const saved = localStorage.getItem('coeg_repeat');
  if (saved === 'all' || saved === 'one' || saved === 'off') return saved;
  return 'off';
};

class PlayerStore {
  songs = $state<Song[]>([]);
  activeQueue = $state<Song[]>([]);
  historyStack = $state<HistoryItem[]>([]); // Menyimpan lagu + konteks queue
  playedInQueue = $state<Set<number>>(new Set());
  currentSong = $state<Song | null>(null);
  quickPicks = $state<Song[]>([]);
  aiQueue = $state<AiQueueItem[]>([]);
  isPlaying = $state(false);
  currentTime = $state(0);
  duration = $state(0);

  volume = $state(getInitialVolume());
  isMuted = $state(getInitialMute());
  isShuffle = $state(getInitialShuffle());
  repeatMode = $state(getInitialRepeat());
  
  private audio = new Audio();
  public analyser: AnalyserNode | null = null;
  private gainNode: GainNode | null = null;
  private audioCtx: AudioContext | null = null;
  private isAudioSourceConnected = false;

  constructor() {
    this.audio.crossOrigin = 'anonymous';
    this.audio.volume = this.volume;
    this.audio.muted = this.isMuted;

    this.audio.onended = () => {
      if (this.repeatMode === 'one') {
        this.audio.currentTime = 0;
        this.audio.play();
      } else {
        this.next();
      }
    };
    this.audio.onpause = () => (this.isPlaying = false);
    this.audio.onplay = () => (this.isPlaying = true);
    this.audio.ontimeupdate = () => {
      this.currentTime = this.audio.currentTime;

      if (typeof window !== 'undefined' && this.currentSong) {
        localStorage.setItem('coeg_last_time', Math.floor(this.audio.currentTime).toString());
      }
    };
   this.audio.onloadedmetadata = () => {
      this.duration = this.audio.duration;

      if (this.currentSong && (!this.currentSong.duration || this.currentSong.duration === 0)) {
        const realDuration = Math.floor(this.audio.duration);
        this.currentSong.duration = realDuration;

        const songInList = this.songs.find((s) => s.id === this.currentSong?.id);
        if (songInList) songInList.duration = realDuration;

        const songInQueue = this.activeQueue.find((s) => s.id === this.currentSong?.id);
        if (songInQueue) songInQueue.duration = realDuration;
      }
    };
  }

  private initWebAudio() {
    if (typeof window === 'undefined') return;

    try {
      if (!this.audioCtx) {
        const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
        this.audioCtx = new AudioContextClass();
        this.analyser = this.audioCtx.createAnalyser();
        this.analyser.fftSize = 64; // 32 frequency bins
        this.analyser.smoothingTimeConstant = 0.75;
        this.analyser.minDecibels = -85;
        this.analyser.maxDecibels = -25;
      }

      if (this.audioCtx && this.audioCtx.state === 'suspended') {
        this.audioCtx.resume();
      }

      if (!this.isAudioSourceConnected && this.analyser && this.audioCtx) {
        this.gainNode = this.audioCtx.createGain();
        const source = this.audioCtx.createMediaElementSource(this.audio);
        source.connect(this.analyser);
        this.analyser.connect(this.gainNode);
        this.gainNode.connect(this.audioCtx.destination);
        this.isAudioSourceConnected = true;
        this.audio.volume = 1;
        this.audio.muted = false;
        this.applyNormalizedVolume();
      }
    } catch (e) {
      console.warn('Web Audio API init warning:', e);
    }
  }

  generateAiQueue() {
    if (!this.currentSong || this.songs.length <= 1) {
      this.aiQueue = [];
      return;
    }

    setTimeout(() => {
      const curr = this.currentSong;
      if (!curr) return;

      const candidates = this.songs.filter((s) => s.id !== curr.id);

      const scored = candidates.map((song) => {
        let score = 40;
        const matchedReasons: { name: string; weight: number }[] = [];

        if (
          song.artist &&
          curr.artist &&
          song.artist.toLowerCase() !== 'unknown artist' &&
          song.artist.toLowerCase() === curr.artist.toLowerCase()
        ) {
          score += 35;
          matchedReasons.push({ name: 'Artist similarity', weight: 35 });
        }

        if (
          song.album &&
          curr.album &&
          song.album.toLowerCase() !== 'unknown album' &&
          song.album.toLowerCase() === curr.album.toLowerCase()
        ) {
          score += 25;
          matchedReasons.push({ name: 'Album continuity', weight: 25 });
        }

        if (
          song.genre &&
          curr.genre &&
          song.genre.toLowerCase() !== 'unknown genre' &&
          song.genre.toLowerCase() === curr.genre.toLowerCase()
        ) {
          score += 20;
          matchedReasons.push({ name: 'Genre continuity', weight: 20 });
        }

        if (song.is_lossless === curr.is_lossless && song.is_lossless !== undefined) {
          score += 10;
          matchedReasons.push({ name: 'Audio profile parity', weight: 10 });
        }

        if (song.duration && curr.duration && Math.abs(song.duration - curr.duration) < 45) {
          score += 5;
          matchedReasons.push({ name: 'Pacing match', weight: 5 });
        }

        matchedReasons.sort((a, b) => b.weight - a.weight);
        const primaryReason = matchedReasons.length > 0 ? matchedReasons[0].name : 'Discovery wildcard';

        return { song, rawScore: score, reason: primaryReason };
      });

      scored.sort((a, b) => b.rawScore - a.rawScore);

      const top10: typeof scored = [];
      const artistCountMap = new Map<string, number>();

      for (const item of scored) {
        if (top10.length >= 10) break;

        const artistKey = (item.song.artist || 'Unknown Artist').toLowerCase().trim();
        const count = artistCountMap.get(artistKey) || 0;

        if (count < 2) {
          top10.push(item);
          artistCountMap.set(artistKey, count + 1);
        }
      }

      if (top10.length < 10 && scored.length > top10.length) {
        for (const item of scored) {
          if (top10.length >= 10) break;
          if (!top10.some((t) => t.song.id === item.song.id)) {
            top10.push(item);
          }
        }
      }

      const balancedQueue = top10.map((item, idx) => {
        let visualScore = 75;

        if (idx < 3) {
          visualScore = 96 - idx * 2 - Math.floor(Math.random() * 2);
        } else if (idx < 7) {
          visualScore = 88 - (idx - 3) * 2 - Math.floor(Math.random() * 2);
        } else {
          visualScore = 78 - (idx - 7) * 3 - Math.floor(Math.random() * 2);
        }

        return {
          song: item.song,
          matchScore: visualScore,
          reason: item.reason
        };
      });

      this.aiQueue = balancedQueue;
    }, 0);
  }

  refreshWildcards() {
    if (!this.currentSong || this.songs.length <= 1 || this.aiQueue.length < 7) return;

    const curr = this.currentSong;
    const top7Ids = new Set(this.aiQueue.slice(0, 7).map((item) => item.song.id));
    top7Ids.add(curr.id);

    const remainingCandidates = this.songs.filter((s) => !top7Ids.has(s.id));
    if (remainingCandidates.length === 0) return;

    const shuffled = [...remainingCandidates].sort(() => 0.5 - Math.random());
    const newWildcards = shuffled.slice(0, 3).map((song, idx) => {
      const visualScore = 78 - idx * 3 - Math.floor(Math.random() * 2);
      return {
        song,
        matchScore: visualScore,
        reason: 'Discovery wildcard'
      };
    });

    this.aiQueue = [...this.aiQueue.slice(0, 7), ...newWildcards];
  }

  refreshQuickPicks() {
    if (!this.songs || this.songs.length === 0) {
      this.quickPicks = [];
      return;
    }
    if (this.songs.length <= 3) {
      this.quickPicks = [...this.songs];
      return;
    }
    
    const shuffled = [...this.songs].sort(() => 0.5 - Math.random());
    this.quickPicks = shuffled.slice(0, 3);
  }

  async play(song?: Song, customQueue?: Song[], isFromHistory = false, forcePlay = false) {
    if (customQueue && customQueue.length > 0) {
      this.activeQueue = customQueue;
      this.playedInQueue.clear();
    } else if (!this.activeQueue || this.activeQueue.length === 0) {
      this.activeQueue = this.songs;
    }

    if (song) {
      if (this.currentSong?.id === song.id && !forcePlay) {
        this.toggle();
        return;
      }

      this.playedInQueue.add(song.id);

      // Simpan lagu + antrean aktifnya ke historyStack
      if (this.currentSong && !isFromHistory && this.currentSong.id !== song.id) {
        this.historyStack.push({
          song: this.currentSong,
          queue: [...this.activeQueue]
        });
      }

      this.currentSong = song;
      const safePath = song.file_path.replace(/\\/g, '/');
      this.audio.src = convertFileSrc(safePath);
      this.applyNormalizedVolume();

      if (typeof window !== 'undefined') {
        localStorage.setItem('coeg_last_song_id', song.id.toString());
        localStorage.setItem('coeg_last_time', '0');
      }

      this.generateAiQueue();
    }

    if (this.currentSong) {
      this.initWebAudio();
      this.audio.play().then(() => {
        this.isPlaying = true;
      }).catch((err) => {
        console.error('Error playing audio:', err);
      });
    }
  }

  async toggle() {
    if (!this.currentSong) return;
    if (this.isPlaying) {
      this.audio.pause();
    } else {
      this.initWebAudio();
      this.audio.play().then(() => {
        this.isPlaying = true;
      }).catch((err) => {
        console.error('Error toggling audio:', err);
      });
    }
  }

  next() {
    let queue = this.activeQueue.length > 0 ? this.activeQueue : this.songs;
    if (!this.currentSong || queue.length === 0) return;

    if (this.activeQueue !== this.songs) {
      const unplayedInAlbum = this.activeQueue.filter((s) => !this.playedInQueue.has(s.id));
      
      if (unplayedInAlbum.length === 0) {
        this.activeQueue = this.songs;
        queue = this.songs;
        this.playedInQueue.clear();
      }
    }

    if (this.isShuffle) {
      let unplayed = queue.filter((s) => !this.playedInQueue.has(s.id));

      if (unplayed.length === 0) {
        this.playedInQueue.clear();
        unplayed = queue;
      }

      let available = unplayed.filter((s) => s.id !== this.currentSong?.id);
      if (available.length === 0) available = unplayed;

      const randomIndex = Math.floor(Math.random() * available.length);
      const nextSong = available[randomIndex];
      
      this.play(nextSong, undefined, false, true);
      return;
    }

    const idx = queue.findIndex((s) => s.id === this.currentSong?.id);
    if (idx !== -1 && idx < queue.length - 1) {
      this.play(queue[idx + 1], undefined, false, true);
    } else if (this.repeatMode === 'all') {
      this.play(queue[0], undefined, false, true);
    } else if (this.activeQueue !== this.songs) {
      const masterIdx = this.songs.findIndex((s) => s.id === this.currentSong?.id);
      this.activeQueue = this.songs;
      if (masterIdx !== -1 && masterIdx < this.songs.length - 1) {
        this.play(this.songs[masterIdx + 1], undefined, false, true);
      } else {
        this.play(this.songs[0], undefined, false, true);
      }
    }
  }

  prev() {
    if (!this.currentSong) return;

    if (this.currentTime > 3) {
      this.seek(0);
      return;
    }

    // Restore histori lagu beserta konteks antreannya
    if (this.historyStack.length > 0) {
      const previous = this.historyStack.pop();
      if (previous) {
        this.play(previous.song, previous.queue, true, true);
        return;
      }
    }

    const queue = this.activeQueue.length > 0 ? this.activeQueue : this.songs;
    const idx = queue.findIndex((s) => s.id === this.currentSong?.id);
    const prevIdx = idx > 0 ? idx - 1 : queue.length - 1;
    this.play(queue[prevIdx], undefined, false, true);
  }

  seek(timeSeconds: number) {
    if (!this.currentSong) return;
    this.audio.currentTime = timeSeconds;
    this.currentTime = timeSeconds;
  }

  setVolume(val: number) {
    this.volume = val;
    this.applyNormalizedVolume();
    if (val > 0) this.isMuted = false;

    if (typeof window !== 'undefined') {
      localStorage.setItem('coeg_volume', val.toString());
      localStorage.setItem('coeg_muted', this.isMuted.toString());
    }
  }

  toggleMute() {
    this.isMuted = !this.isMuted;
    this.applyNormalizedVolume();

    if (typeof window !== 'undefined') {
      localStorage.setItem('coeg_muted', this.isMuted.toString());
    }
  }

  toggleShuffle() {
    this.isShuffle = !this.isShuffle;
    if (typeof window !== 'undefined') {
      localStorage.setItem('coeg_shuffle', this.isShuffle.toString());
    }
  }

  toggleRepeat() {
    if (this.repeatMode === 'off') this.repeatMode = 'all';
    else if (this.repeatMode === 'all') this.repeatMode = 'one';
    else this.repeatMode = 'off';

    if (typeof window !== 'undefined') {
      localStorage.setItem('coeg_repeat', this.repeatMode);
    }
  }

  stop() {
    this.audio.pause();
    this.audio.src = '';
    this.currentSong = null;
    this.isPlaying = false;
    this.currentTime = 0;
    this.duration = 0;
  }

  async toggleFavorite(song: Song) {
    const newStatus = !song.is_favorite;
    song.is_favorite = newStatus;

    // Sync state reaktif di master list & active queue
    const foundInSongs = this.songs.find((s) => s.id === song.id);
    if (foundInSongs) foundInSongs.is_favorite = newStatus;

    const foundInQueue = this.activeQueue.find((s) => s.id === song.id);
    if (foundInQueue) foundInQueue.is_favorite = newStatus;

    // Persist ke database SQLite via IPC Rust
    try {
      await invoke('toggle_favorite', { songId: song.id, isFavorite: newStatus });
    } catch (e) {
      console.error('Gagal simpan status favorite:', e);
    }
  }

  restoreLastSession() {
    if (typeof window === 'undefined' || !this.songs || this.songs.length === 0) return;

    const savedSongId = localStorage.getItem('coeg_last_song_id');
    const savedTime = localStorage.getItem('coeg_last_time');

    if (savedSongId) {
      const songId = parseInt(savedSongId, 10);
      const foundSong = this.songs.find((s) => s.id === songId);

      if (foundSong) {
        this.currentSong = foundSong;
        const safePath = foundSong.file_path.replace(/\\/g, '/');
        this.audio.src = convertFileSrc(safePath);

        if (savedTime) {
          const timeSec = parseFloat(savedTime);
          if (!isNaN(timeSec) && timeSec < foundSong.duration) {
            this.audio.currentTime = timeSec;
            this.currentTime = timeSec;
          }
        }

        this.generateAiQueue();
      }
    }
  }

  private applyNormalizedVolume() {
    let targetVolume = this.volume;

    if (this.isMuted) {
      targetVolume = 0;
    } else {
      const gainDb = this.currentSong?.replay_gain;

      if (gainDb != null && !isNaN(gainDb)) {
        const multiplier = Math.pow(10, gainDb / 20);
        targetVolume = Math.min(1.0, Math.max(0.0, this.volume * multiplier));
      }
    }

    if (this.gainNode) {
      this.gainNode.gain.value = targetVolume;
    } else {
      this.audio.volume = targetVolume;
    }
  }
}

export const player = new PlayerStore();