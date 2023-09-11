import { defineStore } from "pinia";

export const useToastStore = defineStore("toast", {
    state: () => {
        return {
            toastTimeout: null as ReturnType<typeof setTimeout> | null,
            toast: null as Toast | null,
        }
    },
    actions: {
        showToast(message: string, type: ToastType) {
            if (this.toastTimeout) {
                clearTimeout(this.toastTimeout);
            }

            this.toast = {
                message,
                type,
            };
            this.toastTimeout = setTimeout(() => { this.toast = null; }, 5000);
        },
        closeToast() {
            if (this.toastTimeout) {
                clearTimeout(this.toastTimeout);
            }
            this.toast = null;
        }
    },
});


export enum ToastType {
    SUCCESS = 'success',
    ERROR = 'error',
}

interface Toast {
    message: string;
    type: ToastType;
}
