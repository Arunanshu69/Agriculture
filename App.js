import { StatusBar } from 'expo-status-bar';
import React from 'react';
import { StyleSheet, Text, View, SafeAreaView } from 'react-native';
import QRCode from "react-native-qrcode-svg";

export default function App() {
  return (
    <SafeAreaView style={styles.container}>
      <Text>Hello</Text>
      <Text style={{ fontSize: 20, marginBottom: 20 }}>Scan QR Code</Text>
      <QRCode
        value="https://yourwebsite.com"   
        size={200}                        
        color="black"                     
        backgroundColor="white"           
      />
      <StatusBar style="auto" />
    </SafeAreaView>
  );
}
const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
  },
});
