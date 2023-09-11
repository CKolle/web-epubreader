<script lang="ts">
import { RouterView } from 'vue-router';
import { useUserStore } from './stores/user';
import { useNavigationStore } from '@/stores/navigations'
import TheToaster from './components/TheToaster.vue';
import NavbarMenu from './components/NavbarMenu.vue';
import HamburgerMenu from './components/HamburgerMenu.vue';
export default {
  name: "App",
  components: {
    RouterView,
    TheToaster,
    NavbarMenu,
    HamburgerMenu,
  },
  computed: {
    isLoggedin() {
      return useUserStore().isLoggedIn;
    },
    navigationHidden() {
      return useNavigationStore().isHidden;
    }
  },
  mounted() {
    console.log(this.$route.path);
  }
};

</script>

<template>
  <NavbarMenu v-if="!navigationHidden" />
  <HamburgerMenu v-if="!navigationHidden" />
  <TheToaster />
  <router-view />
</template>

