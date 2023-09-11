import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '../views/LoginView.vue'
import HomeView from '../views/HomeView.vue'
import TotalCollectionView from '../views/TotalCollectionView.vue'
import SettingsView from '../views/SettingsView.vue'
import LibrarySettingsView from '../views/settings/LibrarySettingsView.vue'
import BookView from '../views/BookView.vue'
import TheBookReader from '@/components/TheBookReader.vue'
import NotFoundViewVue from '@/views/NotFoundView.vue'
import { useUserStore } from '@/stores/user'
import { useNavigationStore } from '@/stores/navigations'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'about',
      component: HomeView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/login',
      name: 'login',
      component: LoginView,
      meta: {
        hiddenToAuth: true,
        navigationHidden: true
        
      }
    },
    {
      path: "/collection",
      name: "collection",
      component: TotalCollectionView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: "/settings",
      name: "settings",
      component: SettingsView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: "/settings/libraries",
      name: "librarySettings",
      component: LibrarySettingsView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: "/book/:id",
      name: "book",
      component: BookView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: "/read/:id",
      name: "read",
      component: TheBookReader,
      meta: {
        requiresAuth: true,
        navigationHidden: true
      }
    },
    {
      path: "/:pathMatch(.*)*",
      name: "notFound",
      component: NotFoundViewVue,
      meta: {
        requiresAuth: true
      }
    }
  ]
})

router.beforeEach((to) => {
  if (to.meta.requiresAuth && !useUserStore().isLoggedIn) {
    return {
      path: '/login',
      query: { redirect: to.fullPath },
    }
  }

  if (to.meta.hiddenToAuth && useUserStore().isLoggedIn) {
    return {
      path: '/',
    }
  }

  if (to.meta.navigationHidden) {
    useNavigationStore().hide();
  } else {
    useNavigationStore().show();
  }
});

export default router
