<script lang="ts">
	import "../app.css";
	import "$lib/styles/scroll.css";
	import SplashScreen from '$lib/components/SplashScreen.svelte';
	import { onMount } from 'svelte';
	import { invoke } from "@tauri-apps/api/core";
	import { browser } from '$app/environment';
	import { loadFormatsData } from '$lib/data/formats';
	import ScrollContainer from "$lib/components/ScrollContainer.svelte";
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { goto } from '$app/navigation';
	import { setLocale } from '$lib/paraglide/runtime';
	import { page } from '$app/state';

	let { children } = $props();
	const isHome = browser && window.location.pathname === '/';
	const appReady = browser && sessionStorage.getItem('app-ready') === 'true';
	let showSplash = $state(isHome && !appReady);
	let splashDone = $state(appReady);

	$effect(() => {
		const lang = page.data?.settings?.language;
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

	async function setAppBackground() {
		try {
			await invoke('set_window_background', { r: 20, g: 10, b: 41, a: 255 });
		} catch(e) {
			console.warn('Failed to set background:', e);
		}
	}

	async function onSplashComplete() {
		splashDone = true;
		showSplash = false;

		if (browser) {
			sessionStorage.setItem('app-ready', 'true');
		}

		await setAppBackground();
	}
</script>

{#if showSplash && !splashDone}
	<SplashScreen onComplete={onSplashComplete} />
{/if}

<div class="bg-background text-foreground">{@render children?.()}</div>