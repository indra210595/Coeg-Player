export type ViewType = 'library' | 'favorites' | 'artists' | 'albums' | 'genres' | 'playlists';

class ViewStore {
  current = $state<ViewType>('library');

  set(view: ViewType) {
    this.current = view;
  }
}

export const viewStore = new ViewStore();