package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"time"
)

// BidRequest represents a mock OpenRTB bid request.
type BidRequest struct {
	ID   string `json:"id"`
	User struct {
		ID string `json:"id"`
	} `json:"user"`
	Device struct {
		UA string `json:"ua"`
		IP string `json:"ip"`
	} `json:"device"`
}

// BidResponse represents a mock OpenRTB bid response.
type BidResponse struct {
	ID    string  `json:"id"`
	BidID string  `json:"bidid"`
	Price float64 `json:"price"`
}

func main() {
	dspURL := "http://localhost:8001/bid"
	fmt.Println("🚀 Ad Exchange Simulator started...")
	fmt.Printf("🎯 Targeting DSP at %s\n", dspURL)

	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			sendBidRequest(dspURL)
		}
	}
}

func sendBidRequest(url string) {
	req := BidRequest{
		ID: fmt.Sprintf("req-%d", time.Now().Unix()),
	}
	req.User.ID = "user-123"
	req.Device.UA = "Mozilla/5.0"
	req.Device.IP = "1.1.1.1"

	body, _ := json.Marshal(req)
	resp, err := http.Post(url, "application/json", bytes.NewBuffer(body))
	if err != nil {
		log.Printf("❌ Failed to send bid request: %v\n", err)
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusOK {
		var bid BidResponse
		json.NewDecoder(resp.Body).Decode(&bid)
		fmt.Printf("✅ Received Bid: ID=%s, Price=%.2f\n", bid.ID, bid.Price)
		// Simulate a Win Notification
		sendWinNotification(bid.ID, bid.Price)
	} else {
		fmt.Printf("ℹ️  No bid received (HTTP %d)\n", resp.StatusCode)
	}
}

func sendWinNotification(bidID string, price float64) {
	fmt.Printf("🏆 WIN: %s at price %.2f\n", bidID, price)
	// In a real scenario, this would publish to Pub/Sub
}
