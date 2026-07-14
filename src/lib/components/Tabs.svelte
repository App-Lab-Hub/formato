<script lang="ts">
  // Храним активную вкладку ('login' или 'register')
  let activeTab = $state('login');
</script>

<div class="flex justify-center mt-2">
  <!-- Передаем состояние активного таба через data-аттрибут для CSS -->
  <div 
    data-active={activeTab}
    class="tabs-container relative flex items-center w-fit dark:bg-[#1a1c20] bg-muted p-1 text-foreground rounded-md border h-10"
  >
    
    <!-- Единая скользящая плашка заднего фона (её невозможно продублировать кликами) -->
    <div class="tab-indicator absolute top-1 bottom-1 left-1 bg-background dark:bg-card border dark:border-neutral-800 shadow-sm rounded-sm z-10 w-[calc(50%-4px)]"></div>

    <!-- Кнопка Login -->
    <button
      type="button"
      onclick={() => activeTab = 'login'}
      class="relative z-20 px-4 py-1.5 text-sm font-medium uppercase transition-colors duration-200 w-24 text-center rounded-sm"
      class:text-muted-foreground={activeTab !== 'login'}
    >
      <span class="uppercase">Login</span>
    </button>

    <!-- Кнопка Register -->
    <button
      type="button"
      onclick={() => activeTab = 'register'}
      class="relative z-20 px-4 py-1.5 text-sm font-medium uppercase transition-colors duration-200 w-24 text-center rounded-sm"
      class:text-muted-foreground={activeTab !== 'register'}
    >
      <span class="uppercase">Register</span>
    </button>
  </div>
</div>

