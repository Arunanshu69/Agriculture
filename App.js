import { StatusBar } from 'expo-status-bar';
import React from 'react';
import { StyleSheet, Text, View, SafeAreaView, Image } from 'react-native';
import QRCode from "react-native-qrcode-svg";
import { Login } from "./components/Login";
export default function App() {
  return (
    <SafeAreaView style={styles.container}>
      <Login />
      <Text style={{ fontSize: 20, marginBottom: 10 }}>Customer QR Portal</Text>
      <Text>Scan this to get info on Cooperative ID, Crop</Text><Text style={{alignContent: 'center',marginBottom: 20}}>Type and GPS coordinates</Text>
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