import { createApp } from "vue";
import App from "./App.vue";
import { initTheme } from "@/composables/useTheme.ts";

void initTheme();
createApp(App).mount("#app");
