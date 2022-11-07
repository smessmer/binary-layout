//! This module contains an example use case for defining a layout: ICMP packets.

use crate::prelude::*;

// See https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol for ICMP packet layout
define_layout!(icmp_packet, BigEndian, {
  packet_type: u8,
  code: u8,
  checksum: u16,
  rest_of_header: [u8; 4],
  data_section: [u8], // open ended byte array, matches until the end of the packet
});
