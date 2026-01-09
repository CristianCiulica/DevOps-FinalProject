let currentSymbol = 'BTC-USD';
let myChart = null;
let fgChart = null;
let lastPrices = {};
const coinDetails = {
    'BTC-USD': { date: 'Jan 3, 2009', ath: '$126,350.07', atl: '$0.04' },
    'ETH-USD': { date: 'Jul 30, 2015', ath: '$5031.70', atl: '$0.42' },
    'SOL-USD': { date: 'Mar 16, 2020', ath: '$260.06', atl: '$0.50' }
};
let portfolio = { active: false, cash: 0, holdings: { 'BTC-USD': 0, 'ETH-USD': 0, 'SOL-USD': 0 } }

function initPortfolio() {
    const input = document.getElementById('initialBalance');
    const amount = parseFloat(input.value);
    if(isNaN(amount) || amount <= 0) { alert("Please enter a valid amount"); return; }
    portfolio.active = true;
    portfolio.cash = amount;
    document.getElementById('portfolioSetup').style.display = 'none';
    document.getElementById('portfolioLive').style.display = 'block';
    updatePortfolioUI();
}

function resetPortfolio() {
    if(!confirm("Reset demo account?")) return;
    portfolio.active = false;
    portfolio.cash = 0;
    portfolio.holdings = { 'BTC-USD': 0, 'ETH-USD': 0, 'SOL-USD': 0 };
    document.getElementById('portfolioSetup').style.display = 'flex';
    document.getElementById('portfolioLive').style.display = 'none';
}

function executeTrade(type) {
    if(!portfolio.active) { alert("Please create a demo account first (Bottom Left)!"); return; }
    const price = lastPrices[currentSymbol];
    if(!price) { alert("Waiting for price data..."); return; }

    if(type === 'buy') {
        const amountUSD = prompt(`Buying ${currentSymbol} at $${price.toLocaleString()}.\nCash: $${portfolio.cash.toLocaleString()}\nEnter USD amount:`);
        const val = parseFloat(amountUSD);
        if(val && val > 0 && val <= portfolio.cash) {
            const units = val / price;
            portfolio.cash -= val;
            portfolio.holdings[currentSymbol] += units;
            updatePortfolioUI();
        }
    } else {
        const unitsHeld = portfolio.holdings[currentSymbol];
        if(unitsHeld <= 0) { alert("No holdings to sell."); return; }
        const sellUnits = prompt(`Selling ${currentSymbol}.\nHeld: ${unitsHeld.toFixed(4)}\nEnter Units (or 'all'):`, 'all');
        let unitsToSell = sellUnits === 'all' ? unitsHeld : parseFloat(sellUnits);
        if(unitsToSell > 0 && unitsToSell <= unitsHeld) {
            portfolio.holdings[currentSymbol] -= unitsToSell;
            portfolio.cash += unitsToSell * price;
            updatePortfolioUI();
        }
    }
}

function updatePortfolioUI() {
    if(!portfolio.active) return;
    let totalVal = 0;
    const listEl = document.getElementById('holdingsList');
    listEl.innerHTML = "";

    for(const [symbol, units] of Object.entries(portfolio.holdings)) {
        if(units > 0.000001) {
            const price = lastPrices[symbol] || 0;
            const val = units * price;
            totalVal += val;
            listEl.innerHTML += `
                <div class="holding-item">
                    <div><span class="fw-bold text-primary-theme">${symbol.split('-')[0]}</span><div class="small text-muted-theme">${units.toFixed(4)}</div></div>
                    <div class="text-end"><div class="fw-bold text-primary-theme">$${val.toLocaleString('en-US',{maximumFractionDigits:2})}</div></div>
                </div>`;
        }
    }
    if(listEl.innerHTML === "") listEl.innerHTML = '<div class="text-center text-muted mt-3 small opacity-50">No active positions</div>';

    document.getElementById('totalEquity').innerText = "$" + (portfolio.cash + totalVal).toLocaleString('en-US', {minimumFractionDigits: 2});
    document.getElementById('cashBalance').innerText = "Cash: $" + portfolio.cash.toLocaleString('en-US', {minimumFractionDigits: 2});
}

function toggleTheme() {
    const html = document.documentElement;
    const newTheme = html.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    html.setAttribute('data-theme', newTheme);
    updateChartTheme(newTheme);
}

function updateChartTheme(theme) {
    if (!myChart) return;
    const isDark = theme === 'dark';
    myChart.options.scales.y.grid.color = isDark ? 'rgba(255,255,255,0.05)' : 'rgba(0,0,0,0.05)';
    myChart.update('none');
}

function initChart() {
    const ctx = document.getElementById('priceChart').getContext('2d');
    const gradient = ctx.createLinearGradient(0, 0, 0, 350);
    gradient.addColorStop(0, 'rgba(59, 130, 246, 0.2)');
    gradient.addColorStop(1, 'rgba(59, 130, 246, 0)');

    myChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [
                { label: 'Price', data: [], borderColor: '#3b82f6', backgroundColor: gradient, borderWidth: 2, fill: true, tension: 0.4, pointRadius: 0 },
                { label: 'Moving Avg (5)', data: [], borderColor: '#f59e0b', borderDash: [5, 5], borderWidth: 2, fill: false, tension: 0.4, pointRadius: 0 }
            ]
        },
        options: {
            responsive: true, maintainAspectRatio: false,
            plugins: { legend: { display: true, labels: { color: '#94a3b8' } } },
            scales: { x: { display: false }, y: { position: 'right', grid: { color: 'rgba(0,0,0,0.05)' }, ticks: { color: '#94a3b8' } } },
            animation: false
        }
    });
    updateChartTheme(document.documentElement.getAttribute('data-theme'));
}

function initFGChart() {
    const ctx = document.getElementById('fgChart').getContext('2d');
    fgChart = new Chart(ctx, {
        type: 'doughnut',
        data: { labels: ['Val', 'Rest'], datasets: [{ data: [50, 50], backgroundColor: ['#94a3b8', 'rgba(0,0,0,0.1)'], borderWidth: 0, cutout: '85%', circumference: 180, rotation: 270 }] },
        options: { responsive: true, maintainAspectRatio: false, plugins: { legend: { display: false }, tooltip: { enabled: false } }, aspectRatio: 2 }
    });
    fetchFGData();
}

function fetchFGData() {
    fetch('https://api.alternative.me/fng/?limit=1')
        .then(r => r.json())
        .then(d => {
            if (d.data.length > 0) {
                updateFGChart(d.data[0].value, d.data[0].value_classification);
            }
        })
        .catch(err => {
            console.error("Eroare la preluarea Fear & Greed:", err);
            updateFGChart(50, "Neutral");
        });
}

function updateFGChart(val, text) {
    fgChart.data.datasets[0].data = [val, 100 - val];
    let color = val <= 25 ? '#ef4444' : val <= 45 ? '#f59e0b' : val <= 55 ? '#eab308' : val <= 75 ? '#84cc16' : '#10b981';
    fgChart.data.datasets[0].backgroundColor = [color, document.documentElement.getAttribute('data-theme')==='dark'?'#334155':'#e2e8f0'];
    fgChart.update();
    document.getElementById('fgValueDisplay').innerText = val;
    document.getElementById('fgValueDisplay').style.color = color;
    document.getElementById('fgTextDisplay').innerText = text;
}

function connect() {
    const socket = new SockJS('/ws-market');
    const stompClient = Stomp.over(socket);
    stompClient.debug = null;
    stompClient.connect({}, function () {
        stompClient.subscribe('/topic/prices', function (message) {
            handlePriceUpdate(JSON.parse(message.body));
        });
    });
}

function handlePriceUpdate(data) {
    lastPrices[data.symbol] = data.price;
    const sideEl = document.getElementById(`price-${data.symbol}`);
    if (sideEl) {
        sideEl.innerText = "$" + data.price.toLocaleString('en-US', {minimumFractionDigits: 2});
        sideEl.style.color = data.price >= (lastPrices[data.symbol] || 0) ? '#10b981' : '#ef4444';
    }
    if (data.symbol === currentSymbol) {
        document.getElementById('mainPriceDisplay').innerText = "$" + data.price.toLocaleString('en-US', {minimumFractionDigits: 2});
        addDataToChart(data.price, data.averagePrice);

        if(data.isAnomaly) {
            const badge = document.getElementById('simChange');
            badge.className = "badge bg-danger animate-pulse fs-6 mt-2";
            badge.innerText = "⚠️ ANOMALY DETECTED";
            setTimeout(() => {
                badge.className = "badge bg-success bg-opacity-25 text-success fs-6 mt-2";
                badge.innerText = "Live Monitoring";
            }, 2000);
        }
    }
    if(portfolio.active) updatePortfolioUI();
}

function addDataToChart(data, avgPrice) {
    const now = new Date().toLocaleTimeString();
    myChart.data.labels.push(now);
    myChart.data.datasets[0].data.push(data);
    if(avgPrice) {
        myChart.data.datasets[1].data.push(avgPrice);
    } else {
        myChart.data.datasets[1].data.push(data);
    }
    if (myChart.data.labels.length > 50) {
        myChart.data.labels.shift();
        myChart.data.datasets[0].data.shift();
        myChart.data.datasets[1].data.shift();
    }
    myChart.update('none');
}

function updateCoinInfo(symbol) {
    const details = coinDetails[symbol] || { date: '--', ath: '--', atl: '--' };
    document.getElementById('infoATH').innerText = details.ath;
    document.getElementById('infoATL').innerText = details.atl;
    document.getElementById('infoDate').innerText = details.date;
}

function switchCoin(symbol) {
    if (currentSymbol === symbol) return;
    currentSymbol = symbol;
    document.querySelectorAll('.coin-list-item').forEach(el => el.classList.remove('active'));
    event.currentTarget.classList.add('active');
    document.getElementById('chartTitle').innerText = `${symbol.split('-')[0]} Market`;

    const basePrice = symbol.includes('BTC') ? 91000 : symbol.includes('ETH') ? 3100 : 130;
    document.getElementById('statHigh').innerText = (basePrice * 1.05).toLocaleString(undefined, {maximumFractionDigits: 0});
    document.getElementById('statLow').innerText = (basePrice * 0.95).toLocaleString(undefined, {maximumFractionDigits: 0});
    updateCoinInfo(symbol);

    myChart.data.labels = [];
    myChart.data.datasets[0].data = [];
    myChart.data.datasets[1].data = [];
    myChart.update();
    loadHistory(symbol);
}

function loadHistory(symbol) {
    fetch(`/api/prices?symbol=${symbol}`).then(r => r.json()).then(data => {
        data.reverse().forEach(p => {
            myChart.data.labels.push(new Date(p.timestamp).toLocaleTimeString());
            myChart.data.datasets[0].data.push(p.price);
            myChart.data.datasets[1].data.push(p.averagePrice || p.price);
        });
        myChart.update();
        if(data.length) document.getElementById('mainPriceDisplay').innerText = "$" + data[data.length-1].price.toLocaleString();
    });
}

function askAI() {
    const btn = document.getElementById('aiBtn');
    const container = document.getElementById('aiContent');
    const text = document.getElementById('aiText');
    container.style.display = 'block';
    text.innerHTML = '<i class="fa-solid fa-spinner fa-spin"></i> Analyzing market...';
    btn.disabled = true;
    fetch('/api/ai-analysis').then(res => res.text()).then(data => {
        text.innerHTML = "";
        let i = 0;
        function typeWriter() {
            if (i < data.length) { text.innerHTML += data.charAt(i); i++; setTimeout(typeWriter, 15); }
            else { btn.disabled = false; }
        }
        typeWriter();
    }).catch(() => { btn.disabled = false; text.innerHTML = "Error."; });
}

window.onload = function() {
    initChart(); initFGChart(); connect();
    loadHistory('BTC-USD');
    updateCoinInfo('BTC-USD');
};