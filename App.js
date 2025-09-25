import { StatusBar } from 'expo-status-bar';
import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { StyleSheet, Text, View, SafeAreaView, Platform, TextInput, TouchableOpacity, ActivityIndicator, ScrollView } from 'react-native';
import QRCode from "react-native-qrcode-svg";
import { CameraView, useCameraPermissions } from 'expo-camera';
import { Login } from "./components/Login";

function getApiBaseUrl() {
  const envUrl = process.env.EXPO_PUBLIC_API_URL;
  if (envUrl && typeof envUrl === 'string' && envUrl.length > 0) return envUrl;
  if (Platform.OS === 'android') return 'http://10.0.2.2:3000';
  return 'http://127.0.0.1:3000';
}

export default function App() {
  const [permission, requestPermission] = useCameraPermissions();
  const [isScanning, setIsScanning] = useState(false);
  const [scannedText, setScannedText] = useState('');
  const [result, setResult] = useState(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const apiBase = useMemo(() => getApiBaseUrl(), []);

  const handleSubmitToBackend = useCallback(async (data) => {
    setLoading(true);
    setError('');
    setResult(null);
    try {
      const res = await fetch(`${apiBase}/scan`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ data })
      });
      const text = await res.text();
      try {
        const json = JSON.parse(text);
        if (!res.ok) {
          throw new Error(json?.message || text);
        }
        setResult(json);
      } catch (_) {
        if (!res.ok) throw new Error(text);
        setResult({ raw: text });
      }
    } catch (e) {
      setError(String(e?.message || e));
    } finally {
      setLoading(false);
    }
  }, [apiBase]);

  const onBarcodeScanned = useCallback(({ data }) => {
    if (!data) return;
    setIsScanning(false);
    setScannedText(data);
    handleSubmitToBackend(data);
  }, [handleSubmitToBackend]);

  useEffect(() => {
    if (permission && permission.granted) return;
    // Only request permission when user starts scanning to avoid prompts on load
  }, [permission]);

  return (
    <SafeAreaView style={styles.container}>
      <Login />

      <Text style={{ fontSize: 20, marginBottom: 10, marginTop: 20 }}>Customer QR Portal</Text>
      <Text>Scan a QR to retrieve product JSON from backend</Text>

      {/* QR preview example (optional) */}
      <View style={{ marginTop: 16, marginBottom: 16 }}>
        <QRCode
          value="https://example.com/p/example-id"
          size={160}
          color="black"
          backgroundColor="white"
        />
      </View>

      {/* Scanner Controls */}
      <View style={{ width: '100%', maxWidth: 640 }}>
        <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'center', marginBottom: 12 }}>
          <TouchableOpacity
            onPress={async () => {
              setError('');
              if (!permission || !permission.granted) {
                const { granted } = await requestPermission();
                if (!granted) {
                  setError('Camera permission denied');
                  return;
                }
              }
              setIsScanning(true);
              setResult(null);
            }}
            style={{ backgroundColor: '#4a40e2', paddingVertical: 10, paddingHorizontal: 16, borderRadius: 8, marginRight: 12 }}
          >
            <Text style={{ color: '#fff', fontWeight: '600' }}>{isScanning ? 'Scanning…' : 'Start Scan'}</Text>
          </TouchableOpacity>
          {isScanning && (
            <TouchableOpacity
              onPress={() => setIsScanning(false)}
              style={{ backgroundColor: '#999', paddingVertical: 10, paddingHorizontal: 16, borderRadius: 8 }}
            >
              <Text style={{ color: '#fff', fontWeight: '600' }}>Stop</Text>
            </TouchableOpacity>
          )}
        </View>

        {isScanning && (
          <View style={{ width: '100%', height: 260, overflow: 'hidden', borderRadius: 12, borderWidth: 1, borderColor: '#ddd', marginBottom: 12 }}>
            <CameraView
              style={{ flex: 1 }}
              barcodeScannerSettings={{ barcodeTypes: [ 'qr' ] }}
              onBarcodeScanned={onBarcodeScanned}
            />
          </View>
        )}

        {/* Web/paste fallback and manual input */}
        <Text style={{ fontWeight: '600', marginTop: 8, marginBottom: 6 }}>Or paste scanned content (URL/JSON/id):</Text>
        <TextInput
          placeholder={'e.g. https://your-host/p/abc123 or {"id":"abc123"} or abc123'}
          value={scannedText}
          onChangeText={setScannedText}
          multiline
          style={{ borderWidth: 1, borderColor: '#ccc', borderRadius: 8, padding: 10, minHeight: 56 }}
        />
        <View style={{ flexDirection: 'row', marginTop: 8, alignItems: 'center' }}>
          <TouchableOpacity
            onPress={() => handleSubmitToBackend(scannedText)}
            style={{ backgroundColor: '#0a6', paddingVertical: 10, paddingHorizontal: 16, borderRadius: 8, marginRight: 12 }}
          >
            <Text style={{ color: '#fff', fontWeight: '600' }}>Lookup</Text>
          </TouchableOpacity>
          <Text style={{ color: '#666' }}>{apiBase}/scan</Text>
        </View>
      </View>

      {/* Results */}
      <View style={{ width: '100%', maxWidth: 640, marginTop: 16 }}>
        {loading && (
          <View style={{ alignItems: 'center', marginVertical: 12 }}>
            <ActivityIndicator />
            <Text style={{ marginTop: 6 }}>Fetching…</Text>
          </View>
        )}
        {!!error && (
          <Text style={{ color: 'crimson', marginBottom: 8 }}>
            {error}
          </Text>
        )}
        {result && (
          <ScrollView style={{ maxHeight: 220, borderWidth: 1, borderColor: '#eee', borderRadius: 8, padding: 10 }}>
            <Text selectable style={{ fontFamily: Platform.select({ ios: 'Menlo', android: 'monospace' }) }}>
              {JSON.stringify(result, null, 2)}
            </Text>
          </ScrollView>
        )}
      </View>

      <StatusBar style="auto" />
    </SafeAreaView>
  );
}
const styles = StyleSheet.create({
  container: {
    flexGrow: 1,             // ensures it expands and scrolls
    justifyContent: "center",
    alignItems: "center",
    padding: 20,
    backgroundColor: "#fff",
  },
  inner: {
    width: "100%",           // keeps content aligned
    alignItems: "center",
  },
});