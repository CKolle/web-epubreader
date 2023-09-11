<script lang="ts">
import { RouterLink } from 'vue-router';
import { useHamburgerStore } from '@/stores/hamburger';
import { useUserStore } from '@/stores/user';
export default {
    name: "HamburgerMenu",
    components: {
        RouterLink
    },
    computed: {
        isOpen() {
            return useHamburgerStore().isOpen;
        },
        isLoggedin() {
            return useUserStore().isLoggedIn;
        }


    }
}
</script>

<template>
    <div class="hamburger-menu" :class="{ open: isOpen, closed: !isOpen }" v-if="isLoggedin">
        <router-link to="/">
            <div class="item">
                <span class="icon">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-house"
                        viewBox="0 0 16 16">
                        <path
                            d="M8.707 1.5a1 1 0 0 0-1.414 0L.646 8.146a.5.5 0 0 0 .708.708L2 8.207V13.5A1.5 1.5 0 0 0 3.5 15h9a1.5 1.5 0 0 0 1.5-1.5V8.207l.646.647a.5.5 0 0 0 .708-.708L13 5.793V2.5a.5.5 0 0 0-.5-.5h-1a.5.5 0 0 0-.5.5v1.293L8.707 1.5ZM13 7.207V13.5a.5.5 0 0 1-.5.5h-9a.5.5 0 0 1-.5-.5V7.207l5-5 5 5Z" />
                    </svg>
                </span>
                <span class="link-name" :class="{ closed: !isOpen }">Home</span>
            </div>
        </router-link>
        <router-link to="/collection">
            <div class="item">
                <span class="icon">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor"
                        class="bi bi-collection" viewBox="0 0 16 16">
                        <path
                            d="M2.5 3.5a.5.5 0 0 1 0-1h11a.5.5 0 0 1 0 1h-11zm2-2a.5.5 0 0 1 0-1h7a.5.5 0 0 1 0 1h-7zM0 13a1.5 1.5 0 0 0 1.5 1.5h13A1.5 1.5 0 0 0 16 13V6a1.5 1.5 0 0 0-1.5-1.5h-13A1.5 1.5 0 0 0 0 6v7zm1.5.5A.5.5 0 0 1 1 13V6a.5.5 0 0 1 .5-.5h13a.5.5 0 0 1 .5.5v7a.5.5 0 0 1-.5.5h-13z" />
                    </svg>
                </span>

                <span class="link-name" :class="{ closed: !isOpen }">Collection</span>
            </div>

        </router-link>
    </div>
</template>

<style scoped>
@keyframes slideIn {
    0% {
        left: -70px;
    }
}

.hamburger-menu {
    position: fixed;
    left: 0;
    top: 51px;
    height: 100vh;
    background-color: var(--main-colour-light);
    box-sizing: border-box;
    z-index: 100;
}

.open {
    width: 200px;
    animation: slideIn 0.2s ease-in-out;
}

.closed {
    width: 70px;
    transition: width 0.2s ease-in-out;
}

.link-name.closed {
    opacity: 0;
    width: 0;
    margin-left: 0;
    transition: opacity 0.2s ease-in-out;
}

.link-name {
    opacity: 1;
    margin-left: 10px;
    width: unset;
    transition: width 0.2s ease-in-out;
    transition: opacity 0.2s ease-in-out;
}


.item {
    width: 100%;
    height: 50px;
    display: flex;
    justify-content: center;
    align-items: center;
    border-bottom: 1px solid #000;
    box-sizing: border-box;
}

.item:hover {
    background-color: var(--main-colour-dark);
    cursor: pointer;
}

a {
    text-decoration: none;
    color: var(--font-colour);
}
</style>