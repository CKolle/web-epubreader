import { defineStore } from "pinia";


export const useNavigationStore = defineStore("navigation", {
    state: () => {
        return {
            isHidden: false,
        }
    },
    actions: {
        hide() {
            this.isHidden = true;
        },
        show() {
            this.isHidden = false;
        },
    },
});

