<script lang="ts">
import { useUserStore } from '@/stores/user';
import { RouterLink } from 'vue-router';
import { useHamburgerStore } from '@/stores/hamburger';
export default {
    name: "NavbarMenu",
    components: {
        RouterLink
    },
    methods: {
        toggleHamburgerMenu() {
            useHamburgerStore().toggle();

        }
    },
    computed: {
        isLoggedIn() {

            return useUserStore().isLoggedIn;
        }

    },


}
</script>

<template>
    <div class="navbar" v-if="isLoggedIn">
        <div class="head-container">
            <span class="hamburger" @click="toggleHamburgerMenu">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-list"
                    viewBox="0 0 16 16">
                    <path fill-rule="evenodd"
                        d="M2.5 12a.5.5 0 0 1 .5-.5h10a.5.5 0 0 1 0 1H3a.5.5 0 0 1-.5-.5zm0-4a.5.5 0 0 1 .5-.5h10a.5.5 0 0 1 0 1H3a.5.5 0 0 1-.5-.5zm0-4a.5.5 0 0 1 .5-.5h10a.5.5 0 0 1 0 1H3a.5.5 0 0 1-.5-.5z" />
                </svg>
            </span>
            <!-- Todo: Make a logo here using this rando image as a placeholder -->
            <img alt="Logo" width="50" height="50">
        </div>

        <div class="user-nav-container">
            <div class="user-nav">
                <img class="profile-picture" alt="User profile picture" width="50" height="50">
                <span class="username">Username</span>
                <span>
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" fill="currentColor"
                        class="bi bi-chevron-down" viewBox="0 0 16 16">
                        <path fill-rule="evenodd"
                            d="M1.646 4.646a.5.5 0 0 1 .708 0L8 10.293l5.646-5.647a.5.5 0 0 1 .708.708l-6 6a.5.5 0 0 1-.708 0l-6-6a.5.5 0 0 1 0-.708z" />
                    </svg>
                </span>

            </div>

            <RouterLink to="/settings"><span>
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-gear"
                        viewBox="0 0 16 16">
                        <path
                            d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z" />
                        <path
                            d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.42 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.42-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.42-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.115l.094-.319z" />
                    </svg>
                </span>
            </RouterLink>


        </div>
    </div>
</template>

<style scoped>
span {
    display: flex;
    justify-content: center;
    align-items: center;
}


svg {
    color: var(--font-colour);
}

.navbar {
    width: 100%;
    height: 50px;
    position: fixed;
    top: 0;
    left: 0;
    background-color: var(--main-colour-dark);
    display: flex;
    justify-content: space-between;
    border-bottom: 1px solid #000;
    z-index: 100;
}

.user-nav-container {
    display: flex;
    align-items: center;
    margin-right: 2rem;
    justify-content: space-between;
    box-sizing: border-box;
}

.user-nav {
    margin-right: 1rem;
    display: flex;
    gap: 0.5rem;
}

.profile-picture {
    border-radius: 50%;
    max-width: 40px;
    max-height: 40px;

}

.bi-chevron-down:hover {
    transform: rotate(180deg);
    transition: transform 0.2s ease-in-out;
    cursor: pointer;
}

.head-container {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-left: 20px;
}

.hamburger {
    padding: 0.5rem;

}

.hamburger:hover {
    background-color: var(--main-colour);
    border-radius: 50%;
    cursor: pointer;
}
</style>