<script lang="ts">
import { useBookStore } from "@/stores/book";


enum PageDirection {
    Horizontal,
    Vertical,
    None
}

const COLUMN_GAP = 50;

export default {
    name: "TheBookReader",
    data() {
        return {
            book_id: this.$route.params.id as string,
            asset_id: null as string | null,
            book: useBookStore().getBookById(+this.$route.params.id),
            prev_head: document.head.innerHTML,
            page: 0,
            iframekey: 0,
            // The url of the iframe
            loadingUrl: true,
            pageDirection: PageDirection.Horizontal,
            // Used for virtual page turning
            progressPercent: 0,
            // Used for debouncing the resize event
            resizeTimeout: 0,
            prevScrollPercent: 0,
        }
    },
    methods: {
        getPageDirection(isInitial = false) {
            // Vertical and horizontal are swapped because of the column layout
            // IsInitial is used to tell if column layout has been applied
            // Initially it is not


            let iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
            let pageDirection = PageDirection.Horizontal;

            if (iframe && iframe.contentWindow) {
                let scrollHeight = iframe.contentWindow.document.body.scrollHeight;
                let scrollWidth = iframe.contentWindow.document.body.scrollWidth;
                let clientHeight = iframe.contentWindow.document.body.clientHeight;
                let clientWidth = iframe.contentWindow.document.body.clientWidth;

                let widthDiff = scrollWidth - clientWidth;
                let heightDiff = scrollHeight - clientHeight;

                if (clientHeight >= scrollHeight && clientWidth >= scrollWidth) {
                    console.log("Both are bigger");
                    return PageDirection.None;
                }
                console.log("Width Diff: ", widthDiff);
                console.log("Height Diff: ", heightDiff);
                if (widthDiff > heightDiff) {
                    pageDirection = isInitial ? PageDirection.Vertical : PageDirection.Horizontal;
                } else {
                    pageDirection = isInitial ? PageDirection.Horizontal : PageDirection.Vertical;
                }

                console.log("Page Direction: ", pageDirection);
                console.log("Is Initial: ", isInitial);
                console.log(scrollHeight, scrollWidth);
                console.log(clientHeight, clientWidth);
            }

            return pageDirection;
        },

        getMaxScroll() {
            let iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
            if (iframe && iframe.contentWindow) {
                let body = iframe.contentWindow.document.body;
                let isHorizontal = this.pageDirection === PageDirection.Horizontal;

                if (isHorizontal) {
                    return body.scrollWidth - body.clientWidth;
                } else {
                    return body.scrollHeight - body.clientHeight;
                }
            }
            console.error("Tried to get max scrol but iframe was undefined");

        },

        applyBodyNormalisation() {
            let iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
            if (iframe && iframe.contentWindow) {
                let body = iframe.contentWindow.document.querySelector("body");
                if (body) {
                    body.style.setProperty("margin", "0", "important");
                    body.style.setProperty("padding", "0", "important");
                    body.style.setProperty("box-sizing", "border-box", "important");
                }
            }
        },

        applyPagination() {
            let iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
            if (iframe && iframe.contentWindow) {
                // Set the scrollbars to width 0
                // Firefox
                iframe.contentWindow.document.body.style.setProperty("scrollbar-width", "none", "important");
                iframe.contentWindow.document.body.style.setProperty("scrollbar-height", "none", "important");
                // Chrome
                // iframe.contentWindow.document.styleSheets[3].insertRule(`::-webkit-scrollbar { display: none; }`, 0);


                let body = iframe.contentWindow.document.querySelector("body");
                if (body) {
                    body.style.columnWidth = "100vw"
                    body.style.columnGap = COLUMN_GAP + "px";
                    body.style.columnRule = "0px inset #000000";
                    body.style.setProperty("width", "100vw", "important");
                    body.style.setProperty("height", "100vh", "important");
                    body.style.setProperty("max-height", "100vh", "important");
                    body.style.setProperty("max-width", "100vw", "important");
                    body.style.setProperty("overflow-wrap", "break-word", "important");
                }
            }

        },

        calulateClosestScroll(pageScroll: number, maxScroll: number, targetScroll: number) {

            let diff = Infinity;
            let closest = 0;

            // The other way would overshoot
            for (let current = 0; current <= maxScroll; current += pageScroll) {
                let currentDiff = Math.abs(current - targetScroll);
                if (currentDiff < diff) {
                    diff = currentDiff;
                    closest = current;
                }
            }

            return closest;

        },
        calculateScrollPercent() {
            let scroll = 0;
            let iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
            if (!iframe || !iframe.contentWindow) {
                return scroll;
            }
            if (this.pageDirection === PageDirection.None) {
                return scroll;
            }

            const isHorizontal = this.pageDirection === PageDirection.Horizontal;
            let maxScroll = this.getMaxScroll() || 1;
            if (isHorizontal) {
                scroll = iframe.contentWindow.document.body.scrollLeft;
            } else {
                scroll = iframe.contentWindow.document.body.scrollTop;
            }

            console.log("Current scrollpercent: ", scroll / maxScroll);

            return scroll / maxScroll;
        },
        calculateScrollFromPercent(percent: number, maxScroll: number) {
            return percent * maxScroll;
        },

        scrollTo(scroll: number) {
            let direction = this.pageDirection;
            let iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
            let isHorizontal = direction === PageDirection.Horizontal;

            if (iframe && iframe.contentWindow) {
                let maxScroll = this.getMaxScroll();
                if (!maxScroll) {
                    return;
                }
                let singlePageScroll = isHorizontal ? iframe.contentWindow.document.body.offsetWidth + COLUMN_GAP : iframe.contentWindow.document.body.offsetHeight + COLUMN_GAP;

                let closestScroll = this.calulateClosestScroll(singlePageScroll, maxScroll, scroll);

                if (isHorizontal) {
                    iframe.contentWindow.scrollTo({
                        left: closestScroll,
                        behavior: "auto"
                    });
                } else {
                    iframe.contentWindow.scrollTo({
                        top: closestScroll,
                        behavior: "auto"
                    });
                }
            }


        },

        resizeListener(event: Event) {
            const isDebouncing = this.resizeTimeout !== 0;
            console.log("Current timeout: ", this.resizeTimeout);
            clearTimeout(this.resizeTimeout);
            const iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
            if (!iframe || !iframe.contentWindow) {
                return;
            }


            let currentPercent = this.calculateScrollPercent();

            if (!isDebouncing) {
                this.prevScrollPercent = currentPercent;
            }

            console.log("Current scroll percent: ", this.prevScrollPercent);



            this.resizeTimeout = setTimeout(() => {
                console.log("Debounce test");
                console.log("Previous scroll percent: ", this.prevScrollPercent);

                // This is only needed if the page is originally none
                let pageDirection = this.getPageDirection();
                this.pageDirection = pageDirection;


                switch (pageDirection) {
                    case PageDirection.None:
                        break;
                    default:
                        this.applyPagination();
                        break;
                }

                let iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
                if (iframe && iframe.contentWindow) {
                    let maxScroll = this.getMaxScroll();
                    if (!maxScroll) {
                        return;
                    }
                    let targetScroll = this.calculateScrollFromPercent(this.prevScrollPercent, maxScroll);
                    this.scrollTo(targetScroll);
                }
                this.resizeTimeout = 0;


            }, 500);

        },

        // Handles both virtual and physical page turning
        turnPage(turnDirection: number) {
            // Can only turn pages that are bigger than the viewport
            // Smaller pages have no overflowing direction
            console.log("Turning page");
            if (this.pageDirection === PageDirection.None) {
                console.log("Page direction is none");
                this.page += turnDirection;
                return;
            }
            switch (turnDirection) {
                case 1:
                    if (this.shouldTurnNext) {
                        this.page += 1;
                        this.progressPercent = 0;
                        return;
                    }
                    break;
                case -1:
                    if (this.shouldTurnPrev) {
                        this.page -= 1;
                        this.progressPercent = 0;
                        return;
                    }
                    break;
                default:
                    break;
            }
            const iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
            if (!iframe || !iframe.contentWindow) {
                return;
            }

            // Scrolling is inverted because of the column layout
            const isHorizontal = this.pageDirection === PageDirection.Horizontal;
            if (isHorizontal) {
                let singlePageScroll = iframe.contentWindow.document.body.offsetWidth + COLUMN_GAP;
                iframe.contentWindow.scrollBy({
                    left: singlePageScroll * turnDirection,
                    behavior: "auto"
                })
            } else {
                let singlePageScroll = iframe.contentWindow.document.body.offsetHeight + COLUMN_GAP;
                iframe.contentWindow.scrollBy({
                    top: singlePageScroll * turnDirection,
                    behavior: "auto"
                })
            }

            this.progressPercent = this.calculateScrollPercent();


        },


        scrollListener(event: WheelEvent) {
            const scrollDirection = event.deltaY > 0 ? 1 : -1;

            this.turnPage(scrollDirection);
        },

        loadBook(event: Event) {

            // Needs to wait for next tick to get the iframe
            let iframe = this.$refs.iframe as HTMLIFrameElement | undefined;
            iframe?.focus();
            iframe?.contentWindow?.window.addEventListener("keydown", (e) => {
                window.postMessage(e.key, "*");
                console.log(e.key);
                switch (e.key) {
                    case "ArrowRight":
                        this.turnPage(1);
                        break;
                    case "ArrowLeft":
                        this.turnPage(-1);
                        break;
                    default:
                        break;
                }
            });

            iframe?.contentWindow?.window.addEventListener("wheel", this.scrollListener);
            iframe?.contentWindow?.window.addEventListener("resize", this.resizeListener);
            if (iframe && iframe.contentWindow) {
                let imgs = iframe.contentWindow.document.querySelectorAll("img");
                for (let i = 0; i < imgs.length; i++) {
                    const img = imgs[i];
                    img.style.maxHeight = 100 + "vh";
                    img.style.maxWidth = 100 + "vw";
                }
            }

            this.applyBodyNormalisation();
            // Need to be known when doing pagination
            let pageDirection = this.getPageDirection(true);
            this.pageDirection = pageDirection;

            switch (pageDirection) {
                case PageDirection.None:
                    break;
                default:
                    this.applyPagination();
                    break;
            }


            console.log(`Page Direction: ${this.pageDirection.toLocaleString()}`);
        },

        syncProgress() {
            let page_num = this.page;
            // let page_percent = this.page
        }

    },
    async mounted() {
        if (this.book) {
            this.asset_id = this.book.book_asset;
        } else {
            let book = await useBookStore().fetchBook(+this.book_id);
            if (!book) {
                this.$router.push("/404");
                console.error("Couldn't find book");
                return;
            }
            this.book = book;
            this.asset_id = book.book_asset;
        }

        this.loadingUrl = false;

    },

    computed: {
        iframe_src() {
            let route = window.location.origin;
            return `${route}/api/v1/book/${this.asset_id}/page/${this.page}`;
        },
        isPrevTurnable() {
            return this.page > 0;
        },
        shouldTurnPrev() {
            if (this.progressPercent === 0) {
                return true;
            }
            return false;
        },
        shouldTurnNext() {
            if (this.progressPercent === 1) {
                return true;
            }
            return false;
        },
    },

    beforeUnmount() {

    }
}

</script>

<template>
    <div class="controls">
        <button v-on:click="page -= 1" :disabled="!isPrevTurnable">Prev</button>
        <button v-on:click="page += 1">Next</button>
        <button @click="$router.push('/')">Home</button>
    </div>

    <div class="iframe-container" ref="container">
        <!-- Have to set the base to behind one for the id and other for read-->
        <iframe :src="iframe_src" frameborder="0" seamless allowfullscreen="true" ref="iframe" scrolling="no"
            v-on:load="(event) => { loadBook(event) }" :key="iframekey" v-if="!loadingUrl">
        </iframe>
    </div>
</template>

<style scoped>
button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.controls {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 40px;
    background-color: var(--main-colour);
    color: white;
    z-index: 100;
}

.iframe-container {
    position: absolute;
    left: 0;
    margin-top: 40px;
    width: 100vw;
    height: calc(100vh - 40px);
    background-color: white;
    overflow: hidden;
}

iframe {
    position: absolute;
    border: none;
    width: 100%;
    height: 100%;
    padding: 10px 20px;
    box-sizing: border-box;



    top: 0;
    left: 0;
}
</style>