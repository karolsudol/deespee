package main

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"time"

	pb "github.com/karolsudol/deespee/adexchange/proto"
	"google.golang.org/protobuf/proto"
)

func main() {
	dspURL := "http://localhost:8001/bid"
	fmt.Println("🚀 Ad Exchange Simulator (Protobuf) started...")
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
	req := &pb.BidRequest{
		Id: fmt.Sprintf("req-%d", time.Now().Unix()),
		User: &pb.User{
			Id: "user-123",
		},
		Device: &pb.Device{
			Ua: "Mozilla/5.0",
			Ip: "1.1.1.1",
		},
	}

	data, err := proto.Marshal(req)
	if err != nil {
		log.Printf("❌ Failed to marshal protobuf: %v\n", err)
		return
	}

	resp, err := http.Post(url, "application/x-protobuf", bytes.NewBuffer(data))
	if err != nil {
		log.Printf("❌ Failed to send bid request: %v\n", err)
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusOK {
		respData, err := ioutil.ReadAll(resp.Body)
		if err != nil {
			log.Printf("❌ Failed to read response: %v\n", err)
			return
		}

		bidResp := &pb.BidResponse{}
		if err := proto.Unmarshal(respData, bidResp); err != nil {
			log.Printf("❌ Failed to unmarshal response: %v\n", err)
			return
		}

		fmt.Printf("✅ Received Bid: ID=%s, Price=%.2f\n", bidResp.Id, bidResp.Price)
	} else {
		fmt.Printf("ℹ️  No bid received (HTTP %d)\n", resp.StatusCode)
	}
}
