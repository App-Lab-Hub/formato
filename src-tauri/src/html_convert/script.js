document.addEventListener("DOMContentLoaded", () => {
  const toastContainer = document.createElement("div");
  toastContainer.id = "toast-container";
  document.body.appendChild(toastContainer);

  document.querySelectorAll("td").forEach(cell => {
    cell.addEventListener("click", () => {
      const text = cell.innerText.trim();
      if (text !== "") {
        navigator.clipboard.writeText(text).then(() => {
          showToast(
            `Copied: "${text.length > 20 ? text.substring(0, 20) + "..." : text}"`,
          );
        });
      }
    });
  });

  function showToast(message) {
    const toast = document.createElement("div");
    toast.className = "toast-notification";
    toast.innerText = message;
    toastContainer.appendChild(toast);

    setTimeout(() => toast.classList.add("show"), 10);

    setTimeout(() => {
      toast.classList.remove("show");
      toast.addEventListener("transitionend", () => toast.remove());
    }, 2500);
  }
});
