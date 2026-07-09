<!-- src/routes/+layout.svelte -->
<script lang="ts">
	import "../app.css";
	import "$lib/styles/scroll.css";
	import SplashScreen from '$lib/components/SplashScreen.svelte';
	import { onMount } from 'svelte';
	import { invoke } from "@tauri-apps/api/core";
	import { browser } from '$app/environment';
	import { loadFormatsData } from '$lib/data/formats';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { goto } from '$app/navigation';
	import { setLocale } from '$lib/paraglide/runtime';
	import { page } from '$app/state';
	import { applyTheme, watchSystemTheme } from '$lib/data/settings';

	let { children } = $props();
	const isHome = browser && window.location.pathname === '/';
	const appReady = browser && sessionStorage.getItem('app-ready') === 'true';
	let showSplash = $state(isHome && !appReady);
	let splashDone = $state(appReady);

	let lang = $derived(page.data?.settings?.language ?? 'en');
	let settings = $derived(page.data?.settings);

	$effect(() => {
		if (settings?.theme) {
			applyTheme(settings.theme);
		}
	});

	$effect(() => {
		if (lang) {
			setLocale(lang, { reload: false });
		}
	});

	onMount(async () => {
		loadFormatsData();

		getCurrentWebview().listen('navigate', (event) => {
			goto(event.payload as string);
		});

		if (!appReady) {
			await invoke('app_ready').catch(console.error);

			if (!isHome) {
				onSplashComplete();
			}
		}
	});

	onMount(() => {
		if (browser && settings?.theme === 'system' ) {
			const unwatch = watchSystemTheme(() => {
				if (settings?.theme === 'system') {
					applyTheme('system');
				}
			});

			return unwatch;
		}
	});

	async function onSplashComplete() {
		splashDone = true;
		showSplash = false;

		if (browser) {
			sessionStorage.setItem('app-ready', 'true');
		}

		if (settings?.theme) {
			applyTheme(settings.theme);
		}
	}
</script>

{#if showSplash && !splashDone}
	<SplashScreen onComplete={onSplashComplete} />
{/if}

{#key lang}
	<div class="bg-background text-foreground">{@render children?.()}</div>
{/key}