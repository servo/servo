import { permissionTest } from "./permission-helper.js"

const registration = await navigator.serviceWorker.getRegistration();
const params = new URL(import.meta.url).searchParams;
permissionTest(null, params.get("sender"), registration);
