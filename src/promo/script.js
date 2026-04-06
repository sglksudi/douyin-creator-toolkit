document.addEventListener('DOMContentLoaded', () => {
    let currentSlide = 0;
    const slides = document.querySelectorAll('.slide');
    const progressFill = document.querySelector('.progress-fill');
    const totalSlides = slides.length;
    let isAnimating = false;

    // Initialize
    updateSlides();

    // Keyboard Navigation
    document.addEventListener('keydown', (e) => {
        if (isAnimating) return;

        if (e.key === 'ArrowRight' || e.key === ' ' || e.key === 'Enter') {
            if (currentSlide < totalSlides - 1) {
                nextSlide();
            }
        } else if (e.key === 'ArrowLeft') {
            if (currentSlide > 0) {
                prevSlide();
            }
        }
    });

    // Mouse Wheel Navigation
    document.addEventListener('wheel', (e) => {
        if (isAnimating) return;
        if (e.deltaY > 0) {
            if (currentSlide < totalSlides - 1) nextSlide();
        } else {
            if (currentSlide > 0) prevSlide();
        }
    });

    function nextSlide() {
        isAnimating = true;
        currentSlide++;
        updateSlides();
        setTimeout(() => isAnimating = false, 800); // Match CSS transition time roughly
    }

    function prevSlide() {
        isAnimating = true;
        currentSlide--;
        updateSlides();
        setTimeout(() => isAnimating = false, 800);
    }

    function updateSlides() {
        slides.forEach((slide, index) => {
            if (index === currentSlide) {
                slide.classList.add('active');
            } else {
                slide.classList.remove('active');
            }
        });

        // Update Progress Bar
        const progress = ((currentSlide + 1) / totalSlides) * 100;
        progressFill.style.width = `${progress}%`;
    }
});
