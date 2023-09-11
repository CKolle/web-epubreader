import { defineStore } from "pinia";

export const useHamburgerStore = defineStore("hamburger", {
    state: () => {
        return {
            isOpen: false,
        }
    },
    actions: {
        toggle() {
            if (this.isOpen) {
                document.documentElement.style.setProperty('--hamburger-width', 'var(--hamburger-width-closed)');
            }

            if (!this.isOpen) {
                document.documentElement.style.setProperty('--hamburger-width', 'var(--hamburger-width-open)');
            }


            this.isOpen = !this.isOpen;
        }
    },
});