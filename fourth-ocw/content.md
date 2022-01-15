fetch_parsed_dot_price
  从http获取dotPrice并反序列化为struct
fetch_price_info
  调用fetch_parsed_dot_price
  提交到链上
  使用不签名但具签名信息的交易，该笔交易不想支付手续费，因此不签名，但需要知道该笔交易来源是谁
  
