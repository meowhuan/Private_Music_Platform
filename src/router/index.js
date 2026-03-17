import { createRouter, createWebHistory } from "vue-router";
import HomePage from "../pages/HomePage.vue";
import LibraryPage from "../pages/LibraryPage.vue";
import PlaylistsPage from "../pages/PlaylistsPage.vue";
import PlaylistDetailPage from "../pages/PlaylistDetailPage.vue";
import DevicesPage from "../pages/DevicesPage.vue";
import SettingsPage from "../pages/SettingsPage.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "home", component: HomePage },
    { path: "/library", name: "library", component: LibraryPage },
    { path: "/playlists", name: "playlists", component: PlaylistsPage },
    { path: "/playlists/:id", name: "playlist-detail", component: PlaylistDetailPage },
    { path: "/devices", name: "devices", component: DevicesPage },
    { path: "/settings", name: "settings", component: SettingsPage }
  ],
  scrollBehavior() {
    return { top: 0 };
  }
});

export default router;
