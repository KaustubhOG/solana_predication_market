import MarketChart from '@/components/MarketChart'

export default function MarketPage() {
  return (
    <div className="grid grid-cols-2 gap-6">
      {/* LEFT */}
      <div>
        <h2 className="text-lg font-semibold mb-2">
          Market Sentiment (Indicative)
        </h2>
        <MarketChart />
        <p className="text-xs text-gray-400 mt-2">
          Rounded & delayed to preserve privacy
        </p>
      </div>

      {/* RIGHT */}
      <div>
        {/* Bet panel goes here */}
      </div>
    </div>
  )
}
